//! # CalDAV backend
//!
//! The shared-API adapter over a connected [`WebdavClientStd`], whose
//! calendar home-set is already resolved. It projects RFC 4791
//! calendars and calendar object resources onto the shared types.
//!
//! Calendar and item ids are the segments the server returned, decoded
//! where io-webdav reads them back into the same segment, so an id a
//! listing showed addresses the same resource on every verb. The server
//! owns the query, so a [`CalendarTimeRange`] is pushed down as an RFC
//! 4791 `time-range` filter rather than applied locally.
//!
//! [`CalendarTimeRange`]: crate::shared::item::CalendarTimeRange

use anyhow::{Context, Result};
use ical::{
    component::{vevent::VEVENT, vjournal::VJOURNAL, vtodo::VTODO},
    prop::uid::UID,
    tree::cst::IcalCst,
};
use io_webdav::{
    client::{WebdavClientStd, WebdavClientStdError},
    rfc4791::{
        calendar::{CaldavCalendar, CaldavCalendarPatch},
        item::CaldavItemEntry,
    },
    rfc4918::{collection_path, send::WebdavSendError, typed::WebdavUnreadableEntry},
};
use log::warn;
use pimalaya_cli::error::with_code;
use pimalaya_config::secret::SecretResolver;

use crate::{
    caldav::client::{connect, item_refusal, refused, rejects_item, row_code},
    config::CaldavConfig,
    error::{ErrorCode, ItemError, calendar_not_found, recoded},
    shared::{
        calendar::{Calendar, CalendarDiff},
        client::window,
        item::{CalendarItem, CalendarItemList, CalendarItemQuery},
        uuid::uuid_v4,
    },
};

/// The shared-API glue over a connected CalDAV client.
pub struct CaldavBackend {
    client: WebdavClientStd,
}

impl CaldavBackend {
    /// Connects and walks the discovery chain, caching the home-set.
    ///
    /// The credential is resolved through `resolver`, so an account
    /// naming one credential command from several of its backends, or
    /// reopening this one, spawns it once.
    pub fn new(config: CaldavConfig, resolver: &mut SecretResolver) -> Result<Self> {
        Ok(Self {
            client: connect(&config, resolver)?,
        })
    }

    /// Lists every calendar collection under the calendar home-set.
    pub fn list_calendars(&mut self) -> Result<Vec<Calendar>> {
        let calendars = self.client.list_calendars()?;
        Ok(calendars.into_iter().map(calendar_from).collect())
    }

    /// Creates a calendar collection through MKCALENDAR, its URL segment
    /// `id`, else a fresh UUID: a display name may hold what a path cannot.
    pub fn create_calendar(
        &mut self,
        id: Option<&str>,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<String> {
        let id = match id {
            Some(id) => id.to_owned(),
            None => uuid_v4()?,
        };

        let calendar = CaldavCalendar {
            id: id.clone(),
            display_name: Some(name.to_owned()),
            description: description.map(str::to_owned),
            color: color.map(str::to_owned),
            ..Default::default()
        };

        if let Err(err) = self.client.create_calendar(&calendar) {
            return Err(self.mkcalendar_refusal(err, &id));
        }

        // NOTE: a server may answer a MKCALENDAR as made yet drop a
        // property it does not keep, the colour above all: read it back.
        // The calendar is made by then, so a read back that fails is said
        // and the create still succeeds.
        if let Some(color) = color {
            match self.listed(&id) {
                Ok(created) => {
                    if created.and_then(|calendar| calendar.color).is_none() {
                        warn!("the server did not keep the colour {color} of calendar `{id}`");
                    }
                }
                Err(err) => warn!("cannot read calendar `{id}` back for its colour: {err}"),
            }
        }

        Ok(id)
    }

    /// Sets and clears a calendar's properties through PROPPATCH, those
    /// the patch names alone.
    ///
    /// The colour is written first, on its own: a server may keep the
    /// rest of a PROPPATCH while it ignores a colour, so a refused colour
    /// leaves the calendar as it was.
    pub fn update_calendar(&mut self, id: &str, patch: CalendarDiff) -> Result<()> {
        if self.listed(id)?.is_none() {
            return Err(calendar_not_found(format!("Calendar `{id}` not found")));
        }

        if let Some(color) = patch.color {
            let colour = CaldavCalendarPatch {
                id: id.to_owned(),
                color: Some(color),
                ..Default::default()
            };

            self.client.update_calendar(&colour).with_context(|| {
                format!("Write the colour of calendar `{id}`, nothing else written")
            })?;
        }

        let rest = CaldavCalendarPatch {
            id: id.to_owned(),
            display_name: patch.name.map(Some),
            description: patch.description,
            ..Default::default()
        };

        if rest.display_name.is_some() || rest.description.is_some() {
            self.client.update_calendar(&rest)?;
        }

        Ok(())
    }

    /// The failure of a MKCALENDAR of `id`: `calendar-duplicate` when the
    /// server names its `DAV:resource-must-be-null` precondition (RFC 4791
    /// §5.3.1.1), or answers 405 (RFC 4918 §9.3.1, a mapped URL) or 412
    /// and lists a calendar `id`, the server's answer under it; any other
    /// failure as it is.
    ///
    /// A 405 alone could also be a server that makes no calendar at all,
    /// which the listing tells apart.
    fn mkcalendar_refusal(&mut self, err: WebdavClientStdError, id: &str) -> anyhow::Error {
        let duplicate = match refused(&err) {
            Some((_, body)) if body.contains("resource-must-be-null") => true,
            Some((405 | 412, _)) => matches!(self.listed(id), Ok(Some(_))),
            _ => false,
        };

        if !duplicate {
            return err.into();
        }

        let err = anyhow::Error::from(err).context(format!("Calendar `{id}` already exists"));
        with_code(err, |_| Some(ErrorCode::CalendarDuplicate))
    }

    /// The listed calendar `id` addresses, if any.
    ///
    /// Ids compare by the path each addresses under the home set: a listed
    /// id is decoded where it encodes back to the served segment, so the
    /// listed `My Cal` and the served `My%20Cal` name one calendar, and a
    /// segment kept as served (`a%2Fb`) is never found as `a/b`.
    fn listed(&mut self, id: &str) -> Result<Option<CaldavCalendar>, WebdavClientStdError> {
        let listed = self.client.list_calendars()?;
        let home = self.client.calendar_home_set.as_ref();
        let home = home.ok_or(WebdavClientStdError::MissingCalendarHomeSet)?;
        let path = collection_path(home.path(), id);

        Ok(listed
            .into_iter()
            .find(|calendar| collection_path(home.path(), &calendar.id) == path))
    }

    /// Deletes a calendar collection and everything inside it.
    pub fn delete_calendar(&mut self, id: &str) -> Result<()> {
        self.client.delete_calendar(id)?;
        Ok(())
    }

    /// Lists items through a `calendar-query` REPORT.
    ///
    /// A range narrows the query server-side rather than locally. Each row
    /// the server answered without the item's data is an entry, as is a
    /// listing the server truncated, on the first window alone since the
    /// whole calendar is read before the window is cut.
    pub fn list_items(
        &mut self,
        calendar_id: &str,
        query: CalendarItemQuery<'_>,
    ) -> Result<CalendarItemList> {
        let listed = self.client.list_items(calendar_id, &comp_filter(&query))?;

        let items = listed
            .items
            .into_iter()
            .map(|entry| item_from(calendar_id, entry))
            .collect();

        let errors = match query.skip {
            0 => {
                let mut errors: Vec<ItemError> =
                    listed.unreadable.into_iter().map(unreadable_row).collect();
                if listed.truncated {
                    errors.push(truncated());
                }
                errors
            }
            _ => Vec::new(),
        };

        Ok(CalendarItemList {
            items: window(items, query.skip, query.take),
            errors,
        })
    }

    /// Reads one item's raw iCalendar bytes plus its entity tag.
    ///
    /// A 400 is a miss as a 404 is: a GET sends no body to refuse, and
    /// iCloud answers one to a name it does not take (`<uid>` without
    /// `.ics`), which names no item of it.
    pub fn get_item(&mut self, calendar_id: &str, item_id: &str) -> Result<CalendarItem> {
        let body = self
            .client
            .read_item(calendar_id, item_id)
            .map_err(|err| match err {
                WebdavClientStdError::Send(WebdavSendError::HttpStatus { status: 400, .. }) => {
                    recoded(ErrorCode::ItemNotFound, err)
                }
                err => item_refusal(err),
            })
            .with_context(|| format!("Read item `{item_id}` from calendar `{calendar_id}`"))?;

        Ok(CalendarItem {
            id: item_id.to_owned(),
            calendar_id: calendar_id.to_owned(),
            etag: body.etag,
            contents: body.data,
        })
    }

    /// PUTs a new resource.
    ///
    /// A server may name the resource itself, in which case the id it
    /// reported wins over the one asked for. The PUT carries
    /// `If-None-Match: *`, so a 412 answers a name the calendar holds,
    /// derived from the same UID: `item-duplicate`.
    pub fn create_item(&mut self, calendar_id: &str, contents: Vec<u8>) -> Result<String> {
        let id = format!("{}.ics", new_resource_name(&contents));

        match self.client.create_item(calendar_id, &id, contents) {
            Ok(created) => Ok(created.id),
            Err(
                err @ WebdavClientStdError::Send(WebdavSendError::HttpStatus {
                    status: 412, ..
                }),
            ) => Err(recoded(ErrorCode::ItemDuplicate, err).context(format!(
                "Calendar `{calendar_id}` already holds `{id}`; update that item instead"
            ))),
            Err(err) => Err(create_error(err)),
        }
    }

    /// PUTs over an existing resource, optionally gated on `if_match`.
    pub fn update_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<()> {
        self.client
            .update_item(calendar_id, item_id, contents, if_match)
            .map_err(update_error)?;
        Ok(())
    }

    /// DELETEs a resource, gated on `if_match` when given.
    pub fn delete_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        if_match: Option<&str>,
    ) -> Result<()> {
        self.client
            .delete_item(calendar_id, item_id, if_match)
            .map_err(item_refusal)?;
        Ok(())
    }
}

/// The failure of a create: [`invalid_item`]'s hint, else the server's
/// answer.
fn create_error(err: WebdavClientStdError) -> anyhow::Error {
    invalid_item(err).unwrap_or_else(Into::into)
}

/// The failure of an update: [`invalid_item`]'s hint, else
/// [`item_refusal`]'s.
fn update_error(err: WebdavClientStdError) -> anyhow::Error {
    invalid_item(err).unwrap_or_else(item_refusal)
}

/// A hint over the server's answer when it refuses the iCalendar written
/// ([`rejects_item`]), coded `item-invalid` by the same predicate, so the
/// two never disagree; the answer handed back otherwise.
///
/// The server's answer stays the cause, its typed error kept for the
/// readers of [`crate::error::code`].
fn invalid_item(err: WebdavClientStdError) -> Result<anyhow::Error, WebdavClientStdError> {
    let WebdavClientStdError::Send(WebdavSendError::HttpStatus { status, body }) = &err else {
        return Err(err);
    };

    if !rejects_item(*status, body) {
        return Err(err);
    }

    Ok(anyhow::Error::new(err).context(
        "The server rejected the iCalendar as invalid: calendula sends the item unchanged, \
         so the server's answer says what to fix",
    ))
}

/// The VCALENDAR child filter a listing sends.
///
/// A range narrows to VEVENT alone: RFC 4791 9.9 defines the overlap
/// test against a component's own start and end, so a VTODO or
/// VJOURNAL without them would be dropped for the wrong reason.
fn comp_filter(query: &CalendarItemQuery<'_>) -> String {
    let Some(kind) = query.kind else {
        return String::new();
    };

    // NOTE: RFC 4791 9.7.1 nests a time-range inside the component it
    // narrows, so the window travels with the kind. The raw `item`
    // family names no kind and carries no window either, which is why
    // one never escapes without the other.
    let range = query
        .range
        .map(|range| range.to_caldav_filter())
        .unwrap_or_default();

    format!("<C:comp-filter name=\"{}\">{range}</C:comp-filter>", &*kind)
}

/// Projects a CalDAV calendar onto the shared [`Calendar`].
///
/// The collection id stands in when the server sets no display name.
fn calendar_from(calendar: CaldavCalendar) -> Calendar {
    Calendar {
        name: calendar
            .display_name
            .clone()
            .unwrap_or_else(|| calendar.id.clone()),
        id: calendar.id,
        description: calendar.description,
        color: calendar.color,
        default: false,
    }
}

/// Projects a CalDAV multistatus entry onto the shared [`CalendarItem`].
fn item_from(calendar_id: &str, entry: CaldavItemEntry) -> CalendarItem {
    CalendarItem {
        id: entry.id,
        calendar_id: calendar_id.to_owned(),
        etag: entry.etag,
        contents: entry.data,
    }
}

/// The entry of a listed item the server answered without its data: coded
/// from the row's status or the status refusing its properties, a 403
/// `permission-denied`, else `item-unreadable`; each refused property in its
/// sources.
pub(crate) fn unreadable_row(row: WebdavUnreadableEntry) -> ItemError {
    let code = row
        .status
        .into_iter()
        .chain(row.failures.iter().map(|failure| failure.status))
        .find_map(row_code)
        .unwrap_or(ErrorCode::ItemUnreadable);

    let error = match row.status {
        Some(status) => format!("The server refused the item with status {status}"),
        None if !row.failures.is_empty() => String::from("The server refused the item data"),
        None => String::from("The server sent the item without its data"),
    };

    ItemError {
        id: Some(row.id),
        code: Some(code),
        error,
        sources: row.failures.iter().map(ToString::to_string).collect(),
    }
}

/// The entry of a listing the server truncated (a 507 row, RFC 6578
/// §3.6): no item to name and no code, the listing incomplete.
fn truncated() -> ItemError {
    ItemError {
        id: None,
        code: None,
        error: String::from("The server truncated the listing: more items exist than it returned"),
        sources: Vec::new(),
    }
}

/// Derives the resource name a create PUTs to from the item's `UID`.
///
/// A content digest stands in when the bytes carry none. Only a
/// proposal: a server naming the resource itself reports its choice in
/// `Location`, which io-webdav returns instead.
fn new_resource_name(contents: &[u8]) -> String {
    let uid = IcalCst::parse(contents).ok().and_then(|cst| {
        let components = cst.components::<VEVENT>();
        let mut components = components
            .chain(cst.components::<VTODO>())
            .chain(cst.components::<VJOURNAL>());

        components
            .find_map(|component| component.prop::<UID>())
            .map(|uid| uid.0.into_owned())
    });

    match uid {
        Some(uid) if is_safe_segment(&uid) => uid,
        _ => format!("{:016x}", fnv1a(contents)),
    }
}

/// Whether a `UID` can be a URL path segment verbatim.
///
/// No empty value, no separator, nothing needing percent-encoding.
fn is_safe_segment(uid: &str) -> bool {
    !uid.is_empty()
        && uid.len() <= 128
        && uid
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '@'))
}

/// A 64-bit FNV-1a digest, the fallback name for a `UID`-less body.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;

    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    hash
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        io::{ErrorKind, Read, Result as IoResult, Write},
        sync::{Arc, Mutex},
    };

    use ical::component::IcalComponentKind;
    use io_webdav::{client::WebdavStream, rfc4918::WebdavAuth};
    use pimalaya_cli::error::{code_of, with_code};
    use url::Url;

    use super::*;
    use crate::{
        error::{ErrorCode, code},
        shared::item::CalendarTimeRange,
    };

    /// A stream answering its reads with the canned responses, in order,
    /// then reading the end of the stream, or a reset when `reset` is set.
    struct Scripted {
        responses: VecDeque<Vec<u8>>,
        written: Arc<Mutex<Vec<u8>>>,
        reset: bool,
    }

    impl Read for Scripted {
        fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
            let Some(response) = self.responses.front_mut() else {
                return match self.reset {
                    true => Err(ErrorKind::ConnectionReset.into()),
                    false => Ok(0),
                };
            };
            let len = response.len().min(buf.len());
            buf[..len].copy_from_slice(&response[..len]);
            response.drain(..len);
            if response.is_empty() {
                self.responses.pop_front();
            }
            Ok(len)
        }
    }

    impl Write for Scripted {
        fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
            self.written.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> IoResult<()> {
            Ok(())
        }
    }

    /// A backend whose server answers `status` with `body`, its home set
    /// already discovered.
    fn answering(status: u16, body: &str) -> CaldavBackend {
        scripted(&[(status, body)]).0
    }

    /// A backend whose server answers each `(status, body)` in turn, and
    /// what it writes to that server.
    fn scripted(answers: &[(u16, &str)]) -> (CaldavBackend, Arc<Mutex<Vec<u8>>>) {
        let (stream, written) = stream(answers, false);
        (backend(stream), written)
    }

    /// A stream answering each `(status, body)` in turn, then closed as
    /// [`Scripted`] says, and what it is written.
    fn stream(answers: &[(u16, &str)], reset: bool) -> (Scripted, Arc<Mutex<Vec<u8>>>) {
        let response = |(status, body): &(u16, &str)| {
            let len = body.len();
            format!(
                "HTTP/1.1 {status} Answer\r\nContent-Type: application/xml\r\n\
                 Content-Length: {len}\r\n\r\n{body}"
            )
            .into_bytes()
        };
        let written = Arc::new(Mutex::new(Vec::new()));
        let stream = Scripted {
            responses: answers.iter().map(response).collect(),
            written: written.clone(),
            reset,
        };

        (stream, written)
    }

    /// A backend over `stream`, its home set already discovered.
    fn backend(stream: Scripted) -> CaldavBackend {
        let home = Url::parse("https://dav.example.org/calendars/me/").unwrap();
        let mut client = WebdavClientStd::new(stream, WebdavAuth::None, home.clone());
        client.calendar_home_set = Some(home);

        CaldavBackend { client }
    }

    /// The calendar listing of a home holding `cal`, coloured `color`.
    fn calendars(color: Option<&str>) -> String {
        calendar_at("cal", color)
    }

    /// The calendar listing of a home holding the calendar served at the
    /// segment `segment`, coloured `color`.
    fn calendar_at(segment: &str, color: Option<&str>) -> String {
        let color = color
            .map(|color| format!("<a:calendar-color>{color}</a:calendar-color>"))
            .unwrap_or_default();

        format!(
            r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav" xmlns:a="http://apple.com/ns/ical/">
  <d:response>
    <d:href>/calendars/me/{segment}/</d:href>
    <d:propstat>
      <d:prop>
        <d:resourcetype><d:collection/><c:calendar/></d:resourcetype>
        <d:displayname>Cal</d:displayname>
        <c:calendar-description>Old</c:calendar-description>
        {color}
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
</d:multistatus>"#
        )
    }

    /// A PROPPATCH answer acknowledging each property of `props` with
    /// `status`.
    fn patched(props: &[(&str, u16)]) -> String {
        let propstats: String = props
            .iter()
            .map(|(prop, status)| {
                format!(
                    "<d:propstat><d:prop><{prop}/></d:prop>\
                     <d:status>HTTP/1.1 {status} Answer</d:status></d:propstat>"
                )
            })
            .collect();

        format!(
            r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav" xmlns:a="http://apple.com/ns/ical/">
  <d:response><d:href>/calendars/me/cal/</d:href>{propstats}</d:response>
</d:multistatus>"#
        )
    }

    /// The requests a scripted server received, as text.
    fn requests(written: &Arc<Mutex<Vec<u8>>>) -> String {
        String::from_utf8_lossy(&written.lock().unwrap()).into_owned()
    }

    /// A cleared description is removed (`DAV:remove`), never left out of
    /// a set that writes nothing; a new name travels beside it.
    #[test]
    fn a_cleared_description_is_removed() {
        let listing = calendars(None);
        let answer = patched(&[("d:displayname", 200), ("c:calendar-description", 200)]);
        let (mut backend, written) = scripted(&[(207, &listing), (207, &answer)]);

        let patch = CalendarDiff {
            name: Some(String::from("New")),
            description: Some(None),
            color: None,
        };
        backend.update_calendar("cal", patch).unwrap();

        let requests = requests(&written);
        let proppatch = requests.split("PROPPATCH ").nth(1).unwrap_or_default();
        assert!(proppatch.contains("<D:remove>"), "{requests}");
        assert!(proppatch.contains("calendar-description/>"), "{requests}");
        assert!(proppatch.contains("New</D:displayname>"), "{requests}");
        assert_eq!(requests.matches("PROPPATCH ").count(), 1, "{requests}");
    }

    /// The colour goes first and alone: a server ignoring it fails the
    /// update before the name and the description are written.
    #[test]
    fn an_ignored_colour_writes_nothing_else() {
        let listing = calendars(None);
        let (mut backend, written) = scripted(&[(207, &listing), (207, &patched(&[]))]);

        let patch = CalendarDiff {
            name: Some(String::from("New")),
            description: Some(Some(String::from("Text"))),
            color: Some(Some(String::from("#123456"))),
        };
        let err = backend.update_calendar("cal", patch).unwrap_err();
        assert!(format!("{err:#}").contains("calendar-color"), "{err:#}");
        assert!(err.to_string().contains("nothing else written"), "{err}");

        let requests = requests(&written);
        assert_eq!(requests.matches("PROPPATCH ").count(), 1, "{requests}");
        let proppatch = requests.split("PROPPATCH ").nth(1).unwrap_or_default();
        assert!(proppatch.contains("#123456"), "{requests}");
        assert!(!proppatch.contains("displayname"), "{requests}");
        assert!(!proppatch.contains("Text"), "{requests}");
    }

    /// A colour the server drops on create is said, the calendar made.
    #[test]
    fn a_colour_dropped_on_create_is_read_back() {
        let listing = calendars(None);
        let (mut backend, written) = scripted(&[(201, ""), (207, &listing)]);

        let id = backend
            .create_calendar(Some("cal"), "Cal", None, Some("#00ff00"))
            .unwrap();
        assert_eq!(id, "cal");

        let requests = requests(&written);
        assert!(requests.starts_with("MKCALENDAR "), "{requests}");
        assert_eq!(requests.matches("PROPFIND ").count(), 1, "{requests}");
    }

    /// A MKCALENDAR naming a calendar the server holds fails
    /// `calendar-duplicate`, the server's answer in `sources`: a body
    /// naming `resource-must-be-null`, or a 405 the listing confirms.
    #[test]
    fn a_calendar_created_twice_is_a_duplicate() {
        let precondition = r#"<?xml version="1.0"?>
<d:error xmlns:d="DAV:"><d:resource-must-be-null/></d:error>"#;
        let listing = calendars(None);

        let answers: [&[(u16, &str)]; 2] = [&[(403, precondition)], &[(405, ""), (207, &listing)]];

        for answers in answers {
            let (mut backend, _) = scripted(answers);
            let err = backend
                .create_calendar(Some("cal"), "Cal", None, None)
                .unwrap_err();
            let err = with_code(err, code);

            assert_eq!(code_of(&err), Some(ErrorCode::CalendarDuplicate), "{err:#}");
            assert_eq!(err.to_string(), "Calendar `cal` already exists");
            assert!(err.chain().count() > 1, "{err:#}");
        }
    }

    /// A 405 the listing does not confirm is no duplicate: a server that
    /// makes no calendar answers it too.
    #[test]
    fn a_refused_mkcalendar_of_a_new_calendar_is_no_duplicate() {
        let listing = calendar_at("other", None);
        let (mut backend, _) = scripted(&[(405, ""), (207, &listing)]);

        let err = backend
            .create_calendar(Some("cal"), "Cal", None, None)
            .unwrap_err();
        let err = with_code(err, code);

        assert_ne!(code_of(&err), Some(ErrorCode::CalendarDuplicate), "{err:#}");
    }

    const APPLE_ICAL: &str = "http://apple.com/ns/ical/";
    const INFIT: &str = "http://inf-it.com/ns/ab/";

    /// A colour is written as Apple's `calendar-color`, the one iCloud keeps,
    /// never as the inf-it one: on create, and on update, set or cleared.
    #[test]
    fn a_colour_is_written_in_apples_namespace() {
        let listing = calendars(Some("#00ff00"));
        let (mut backend, written) = scripted(&[(201, ""), (207, &listing)]);
        backend
            .create_calendar(Some("cal"), "Cal", None, Some("#00ff00"))
            .unwrap();

        let created = requests(&written);
        let mkcalendar = created.split("PROPFIND ").next().unwrap_or_default();
        assert!(mkcalendar.contains(APPLE_ICAL), "{created}");
        assert!(!mkcalendar.contains(INFIT), "{created}");

        for color in [Some(String::from("#123456")), None] {
            let listing = calendars(None);
            let answer = patched(&[("a:calendar-color", 200)]);
            let (mut backend, written) = scripted(&[(207, &listing), (207, &answer)]);

            let patch = CalendarDiff {
                color: Some(color.clone()),
                ..Default::default()
            };
            backend.update_calendar("cal", patch).unwrap();

            let requests = requests(&written);
            let proppatch = requests.split("PROPPATCH ").nth(1).unwrap_or_default();
            assert!(proppatch.contains(APPLE_ICAL), "{color:?}: {requests}");
            assert!(!proppatch.contains(INFIT), "{color:?}: {requests}");
            assert!(
                proppatch.contains("calendar-color"),
                "{color:?}: {requests}"
            );
        }
    }

    /// A clear goes through io-webdav's client, which sends it again on a
    /// fresh connection when the server closed the one the listing used.
    #[test]
    fn a_clear_is_sent_again_on_a_fresh_connection() {
        let listing = calendars(None);
        let (closing, _) = stream(&[(207, &listing)], true);
        let mut backend = backend(closing);

        let answer = patched(&[("c:calendar-description", 200)]);
        let (fresh, written) = stream(&[(207, &answer)], false);
        let mut fresh = Some(fresh);
        backend.client.set_reconnect(Box::new(move || {
            let fresh = fresh.take().ok_or(ErrorKind::NotConnected)?;
            Ok(Box::new(fresh) as Box<dyn WebdavStream>)
        }));

        let patch = CalendarDiff {
            description: Some(None),
            ..Default::default()
        };
        backend.update_calendar("cal", patch).unwrap();

        let requests = requests(&written);
        assert!(requests.starts_with("PROPPATCH "), "{requests}");
        assert!(requests.contains("<D:remove>"), "{requests}");
    }

    /// A listed calendar id is decoded, and a calendar is found by its listed
    /// id and by the segment the server serves alike, both addressing that
    /// segment; a decoded `/` would address another path, so it is not.
    #[test]
    fn a_calendar_is_found_by_either_spelling() {
        let listing = calendar_at("My%20Cal", None);
        let listed = answering(207, &listing).list_calendars().unwrap();
        assert_eq!(listed[0].id, "My Cal");

        for id in ["My Cal", "My%20Cal"] {
            let answer = patched(&[("d:displayname", 200)]);
            let (mut backend, written) = scripted(&[(207, &listing), (207, &answer)]);

            let patch = CalendarDiff {
                name: Some(String::from("New")),
                ..Default::default()
            };
            backend.update_calendar(id, patch).unwrap();

            let requests = requests(&written);
            let proppatch = requests.split("PROPPATCH ").nth(1).unwrap_or_default();
            assert!(
                proppatch.starts_with("/calendars/me/My%20Cal/ "),
                "{id}: {requests}"
            );
        }

        let listing = calendar_at("a%2Fb", None);
        let patch = CalendarDiff {
            name: Some(String::from("New")),
            ..Default::default()
        };
        let err = answering(207, &listing)
            .update_calendar("a/b", patch)
            .unwrap_err();
        assert_eq!(code(&err), Some(ErrorCode::CalendarNotFound), "{err:#}");
    }

    /// A listed item id is decoded when it encodes back to the segment the
    /// server serves, kept as served otherwise.
    #[test]
    fn a_listed_item_id_is_decoded() {
        let listing = LISTING
            .replace("cal/e1.ics", "cal/My%20Event.ics")
            .replace("cal/e2.ics", "cal/a%2Fb.ics");
        let listed = answering(207, &listing)
            .list_items("cal", CalendarItemQuery::default())
            .unwrap();

        assert_eq!(listed.items[0].id, "My Event.ics");
        assert_eq!(listed.errors[0].id.as_deref(), Some("a%2Fb.ics"));
    }

    /// A `DAV:error` body naming the precondition `name` of `namespace`.
    fn precondition(namespace: &str, name: &str) -> String {
        format!(
            "<?xml version=\"1.0\"?><D:error xmlns:D=\"DAV:\" xmlns:C=\"{namespace}\">\
             <C:{name}/></D:error>"
        )
    }

    const CALDAV: &str = "urn:ietf:params:xml:ns:caldav";

    const EVENT: &[u8] = b"BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\n\
        BEGIN:VEVENT\r\nUID:e1\r\nDTSTAMP:20260101T000000Z\r\n\
        DTSTART:20260814T090000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

    #[test]
    fn a_refused_write_carries_its_code() {
        let need_privileges = precondition("DAV:", "need-privileges");
        let quota = precondition("DAV:", "quota-not-exceeded");
        let data = precondition(CALDAV, "valid-calendar-data");

        let cases = [
            (403, "", Some(ErrorCode::PermissionDenied)),
            (
                403,
                need_privileges.as_str(),
                Some(ErrorCode::PermissionDenied),
            ),
            (403, data.as_str(), Some(ErrorCode::ItemInvalid)),
            (507, "", Some(ErrorCode::QuotaExceeded)),
            (403, quota.as_str(), Some(ErrorCode::QuotaExceeded)),
            (429, "", Some(ErrorCode::RateLimited)),
            (404, "", None),
            (500, "", None),
        ];

        for (status, body, expected) in cases {
            let err = answering(status, body)
                .create_item("cal", EVENT.to_vec())
                .unwrap_err();
            assert_eq!(code(&err), expected, "{status} {body}: {err:#}");
        }
    }

    /// An iCalendar the server refuses is a matter of the item, not of
    /// rights: coded `item-invalid` under a hint, the server's answer its
    /// cause, whether the refusal names a precondition on the content (RFC
    /// 4791 §5.3.2.1) or the iCalendar in plain text, on a create as on an
    /// update. Another precondition carries no code.
    #[test]
    fn a_refused_icalendar_is_invalid_and_keeps_the_answer_as_its_cause() {
        let mut refusals: Vec<(u16, String)> = [
            "supported-calendar-data",
            "valid-calendar-data",
            "valid-calendar-object-resource",
            "supported-calendar-component",
        ]
        .iter()
        .map(|name| (403, precondition(CALDAV, name)))
        .collect();
        refusals.push((400, precondition(CALDAV, "valid-calendar-data")));
        refusals.push((403, String::from("Invalid iCalendar: missing DTSTART")));
        refusals.push((400, String::from("Cannot parse VCALENDAR")));

        for (status, body) in &refusals {
            for update in [false, true] {
                let mut backend = answering(*status, body);
                let err = match update {
                    false => backend.create_item("cal", EVENT.to_vec()),
                    true => backend
                        .update_item("cal", "e1.ics", EVENT.to_vec(), None)
                        .map(|()| String::new()),
                }
                .unwrap_err();

                assert_eq!(code(&err), Some(ErrorCode::ItemInvalid), "{body}: {err:#}");
                assert!(
                    err.to_string()
                        .starts_with("The server rejected the iCalendar")
                );
                let cause = err.chain().nth(1).unwrap().to_string();
                assert!(cause.contains(&status.to_string()), "{cause}");
                let coded = code_of(&with_code(err, code));
                assert_eq!(coded, Some(ErrorCode::ItemInvalid));
            }
        }

        let other = precondition(CALDAV, "max-resource-size");
        let err = answering(403, &other)
            .create_item("cal", EVENT.to_vec())
            .unwrap_err();
        assert_eq!(code(&err), None, "{err:#}");
    }

    #[test]
    fn a_duplicate_uid_and_a_stale_version_carry_their_code() {
        let conflict = precondition(CALDAV, "no-uid-conflict");
        let err = answering(409, &conflict)
            .create_item("cal", EVENT.to_vec())
            .unwrap_err();
        assert_eq!(code(&err), Some(ErrorCode::ItemDuplicate), "{err:#}");

        // NOTE: the name is the UID, so a 412 to the create's
        // `If-None-Match: *` is a UID the calendar holds.
        let err = answering(412, "")
            .create_item("cal", EVENT.to_vec())
            .unwrap_err();
        assert_eq!(code(&err), Some(ErrorCode::ItemDuplicate), "{err:#}");
        assert_eq!(
            err.to_string(),
            "Calendar `cal` already holds `e1.ics`; update that item instead"
        );
        assert!(format!("{err:#}").contains("HTTP 412"), "{err:#}");

        let err = answering(412, "")
            .update_item("cal", "e1.ics", EVENT.to_vec(), Some("\"1\""))
            .unwrap_err();
        assert_eq!(code(&err), Some(ErrorCode::PreconditionFailed), "{err:#}");
    }

    #[test]
    fn an_item_the_server_does_not_hold_is_not_found() {
        for status in [404, 410] {
            let err = answering(status, "")
                .get_item("cal", "nope.ics")
                .unwrap_err();
            assert_eq!(code(&err), Some(ErrorCode::ItemNotFound), "{err:#}");
            assert!(format!("{err:#}").contains("HTTP"), "{err:#}");
        }

        let err = answering(404, "")
            .delete_item("cal", "nope.ics", None)
            .unwrap_err();
        assert_eq!(code(&err), Some(ErrorCode::ItemNotFound), "{err:#}");
    }

    /// iCloud answers 400, not 404, to a GET of a name without `.ics`:
    /// a read names no item then. A write answered 400 naming nothing keeps
    /// no code.
    #[test]
    fn a_read_answered_400_is_not_found_and_a_write_is_not() {
        let err = answering(400, "").get_item("cal", "nope").unwrap_err();
        assert_eq!(code(&err), Some(ErrorCode::ItemNotFound), "{err:#}");
        assert_eq!(err.to_string(), "Read item `nope` from calendar `cal`");
        assert!(format!("{err:#}").contains("HTTP 400"), "{err:#}");

        let err = answering(400, "")
            .update_item("cal", "e1.ics", EVENT.to_vec(), None)
            .unwrap_err();
        assert_eq!(code(&err), None, "{err:#}");
    }

    /// A listing answering an item, a row refused whole, a row refusing its
    /// data, a row missing it and a truncation row.
    const LISTING: &str = r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:response>
    <d:href>/calendars/me/cal/e1.ics</d:href>
    <d:propstat>
      <d:prop>
        <d:getetag>"1"</d:getetag>
        <c:calendar-data>BEGIN:VCALENDAR&#13;
VERSION:2.0&#13;
PRODID:-//x//y//EN&#13;
BEGIN:VEVENT&#13;
UID:e1&#13;
DTSTAMP:20260101T000000Z&#13;
DTSTART:20260814T090000Z&#13;
END:VEVENT&#13;
END:VCALENDAR&#13;
</c:calendar-data>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
  <d:response>
    <d:href>/calendars/me/cal/e2.ics</d:href>
    <d:status>HTTP/1.1 403 Forbidden</d:status>
  </d:response>
  <d:response>
    <d:href>/calendars/me/cal/e3.ics</d:href>
    <d:propstat>
      <d:prop><d:getetag>"3"</d:getetag></d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
    <d:propstat>
      <d:prop><c:calendar-data/></d:prop>
      <d:status>HTTP/1.1 403 Forbidden</d:status>
    </d:propstat>
  </d:response>
  <d:response>
    <d:href>/calendars/me/cal/e4.ics</d:href>
    <d:propstat>
      <d:prop><d:getetag>"4"</d:getetag></d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
  <d:response>
    <d:href>/calendars/me/cal/</d:href>
    <d:status>HTTP/1.1 507 Insufficient Storage</d:status>
  </d:response>
</d:multistatus>"#;

    /// Each row the server refused is an entry naming its item, a 403
    /// `permission-denied`, its refused properties in its sources; the
    /// truncation an entry naming no item and carrying no code.
    #[test]
    fn a_caldav_listing_names_the_rows_it_could_not_read() {
        let listed = answering(207, LISTING)
            .list_items("cal", CalendarItemQuery::default())
            .unwrap();

        let ids: Vec<&str> = listed.items.iter().map(|item| item.id.as_str()).collect();
        assert_eq!(ids, ["e1.ics"]);

        let entries: Vec<_> = listed
            .errors
            .iter()
            .map(|entry| {
                let id = entry.id.as_deref();
                let sources: Vec<&str> = entry.sources.iter().map(String::as_str).collect();
                (id, entry.code, entry.error.as_str(), sources)
            })
            .collect();

        assert_eq!(
            entries,
            [
                (
                    Some("e2.ics"),
                    Some(ErrorCode::PermissionDenied),
                    "The server refused the item with status 403",
                    vec![],
                ),
                (
                    Some("e3.ics"),
                    Some(ErrorCode::PermissionDenied),
                    "The server refused the item data",
                    vec!["calendar-data (403)"],
                ),
                (
                    Some("e4.ics"),
                    Some(ErrorCode::ItemUnreadable),
                    "The server sent the item without its data",
                    vec![],
                ),
                (
                    None,
                    None,
                    "The server truncated the listing: more items exist than it returned",
                    vec![],
                ),
            ]
        );
    }

    /// The whole calendar is read before the window is cut, so the entries
    /// print on the first window only.
    #[test]
    fn a_caldav_listing_names_its_rows_on_the_first_window_only() {
        let query = CalendarItemQuery {
            skip: 1,
            take: Some(2),
            ..Default::default()
        };
        let listed = answering(207, LISTING).list_items("cal", query).unwrap();

        assert!(listed.items.is_empty());
        assert!(listed.errors.is_empty(), "{:?}", listed.errors);
    }

    /// A listed item is the whole body the server sent, the final CRLF it
    /// escapes as `&#13;` included, as a GET returns it.
    #[test]
    fn a_caldav_listing_keeps_the_whole_item() {
        let listed = answering(207, LISTING)
            .list_items("cal", CalendarItemQuery::default())
            .unwrap();

        assert_eq!(listed.items[0].contents, EVENT);
    }

    #[test]
    fn naming_no_kind_asks_for_every_component_kind() {
        assert_eq!(comp_filter(&CalendarItemQuery::default()), "");
    }

    #[test]
    fn a_kind_narrows_the_query_and_the_window_nests_inside_it() {
        let query = |kind, range| CalendarItemQuery {
            kind: Some(kind),
            range,
            ..Default::default()
        };

        assert_eq!(
            comp_filter(&query(IcalComponentKind::VTodo, None)),
            "<C:comp-filter name=\"VTODO\"></C:comp-filter>"
        );

        let range = CalendarTimeRange {
            start: Some("20260801T000000Z".into()),
            end: Some("20260901T000000Z".into()),
        };

        assert_eq!(
            comp_filter(&query(IcalComponentKind::VEvent, Some(&range))),
            "<C:comp-filter name=\"VEVENT\">\
             <C:time-range start=\"20260801T000000Z\" end=\"20260901T000000Z\" />\
             </C:comp-filter>"
        );
    }

    #[test]
    fn a_resource_name_comes_from_the_uid_when_it_is_url_safe() {
        let calendar = concat!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\n",
            "BEGIN:VEVENT\r\nUID:event-1@example.org\r\nDTSTAMP:20260101T000000Z\r\n",
            "DTSTART:20260814T090000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
        );

        assert_eq!(
            new_resource_name(calendar.as_bytes()),
            "event-1@example.org"
        );
    }

    /// A to-do and a journal entry are named after their UID as an event
    /// is, never by a digest.
    #[test]
    fn a_todo_or_a_journal_is_named_after_its_uid() {
        for kind in ["VTODO", "VJOURNAL"] {
            let calendar = format!(
                "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\nBEGIN:{kind}\r\n\
                 UID:todo-1\r\nDTSTAMP:20260101T000000Z\r\nEND:{kind}\r\nEND:VCALENDAR\r\n"
            );

            assert_eq!(new_resource_name(calendar.as_bytes()), "todo-1", "{kind}");
        }
    }

    #[test]
    fn an_unsafe_or_missing_uid_falls_back_to_a_digest() {
        let slashed = concat!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\n",
            "BEGIN:VEVENT\r\nUID:a/b\r\nDTSTAMP:20260101T000000Z\r\n",
            "DTSTART:20260814T090000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
        );

        let name = new_resource_name(slashed.as_bytes());
        assert_eq!(name.len(), 16);
        assert!(name.chars().all(|c| c.is_ascii_hexdigit()));

        assert_eq!(name, new_resource_name(slashed.as_bytes()));
        assert_ne!(name, new_resource_name(b"other bytes entirely"));
    }
}
