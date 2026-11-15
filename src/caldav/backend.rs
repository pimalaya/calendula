//! # CalDAV backend
//!
//! The shared-API adapter over a connected [`WebdavClientStd`], whose
//! calendar home-set is already resolved. It projects RFC 4791
//! calendars and calendar object resources onto the shared types.
//!
//! Item ids are the resource names the server returned verbatim, so an
//! id a listing showed addresses the same resource on every verb. The
//! server owns the query, so a [`CalendarTimeRange`] is pushed down as
//! an RFC 4791 `time-range` filter rather than applied locally.
//!
//! [`CalendarTimeRange`]: crate::shared::item::CalendarTimeRange

use anyhow::{Context, Result};
use ical::{component::vevent::VEVENT, prop::uid::UID, tree::cst::IcalCst};
use io_webdav::{
    client::WebdavClientStd,
    rfc4791::{calendar::CaldavCalendar, item::CaldavItemEntry},
    rfc4918::typed::WebdavUnreadableEntry,
};
use pimalaya_config::secret::SecretResolver;

use crate::{
    caldav::client::{connect, item_refusal, row_code},
    config::CaldavConfig,
    error::{ErrorCode, ItemError, calendar_not_found},
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

        self.client.create_calendar(&calendar)?;
        Ok(id)
    }

    /// Rewrites a calendar's properties through PROPPATCH.
    ///
    /// Reads the current ones first, so a field the patch leaves
    /// untouched is written back unchanged.
    pub fn update_calendar(&mut self, id: &str, patch: CalendarDiff) -> Result<()> {
        let mut calendar = self
            .client
            .list_calendars()?
            .into_iter()
            .find(|calendar| calendar.id == id)
            .ok_or_else(|| calendar_not_found(format!("Calendar `{id}` not found")))?;

        if let Some(name) = patch.name {
            calendar.display_name = Some(name);
        }

        if let Some(description) = patch.description {
            calendar.description = description;
        }

        if let Some(color) = patch.color {
            calendar.color = color;
        }

        self.client.update_calendar(&calendar)?;
        Ok(())
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
    pub fn get_item(&mut self, calendar_id: &str, item_id: &str) -> Result<CalendarItem> {
        let body = self
            .client
            .read_item(calendar_id, item_id)
            .map_err(item_refusal)
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
    /// reported wins over the one asked for.
    pub fn create_item(&mut self, calendar_id: &str, contents: Vec<u8>) -> Result<String> {
        let id = format!("{}.ics", new_resource_name(&contents));
        let created = self.client.create_item(calendar_id, &id, contents)?;
        Ok(created.id)
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
            .map_err(item_refusal)?;
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
fn unreadable_row(row: WebdavUnreadableEntry) -> ItemError {
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
        cst.components::<VEVENT>()
            .next()
            .and_then(|vevent| vevent.prop::<UID>())
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
        io::{Read, Result as IoResult, Write},
    };

    use ical::component::IcalComponentKind;
    use io_webdav::rfc4918::WebdavAuth;
    use url::Url;

    use super::*;
    use crate::{
        error::{ErrorCode, code},
        shared::item::CalendarTimeRange,
    };

    /// A stream answering its reads with the canned responses, in order.
    struct Scripted {
        responses: VecDeque<Vec<u8>>,
    }

    impl Read for Scripted {
        fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
            let Some(response) = self.responses.front_mut() else {
                return Ok(0);
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
            Ok(buf.len())
        }

        fn flush(&mut self) -> IoResult<()> {
            Ok(())
        }
    }

    /// A backend whose server answers `status` with `body`, its home set
    /// already discovered.
    fn answering(status: u16, body: &str) -> CaldavBackend {
        let len = body.len();
        let response = format!(
            "HTTP/1.1 {status} Answer\r\nContent-Type: application/xml\r\n\
             Content-Length: {len}\r\n\r\n{body}"
        );
        let stream = Scripted {
            responses: VecDeque::from([response.into_bytes()]),
        };
        let home = Url::parse("https://dav.example.org/calendars/me/").unwrap();
        let mut client = WebdavClientStd::new(stream, WebdavAuth::None, home.clone());
        client.calendar_home_set = Some(home);

        CaldavBackend { client }
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
            (403, data.as_str(), None),
            (507, "", Some(ErrorCode::QuotaExceeded)),
            (403, quota.as_str(), Some(ErrorCode::QuotaExceeded)),
            (429, "", Some(ErrorCode::RateLimited)),
            (500, "", None),
        ];

        for (status, body, expected) in cases {
            let err = answering(status, body)
                .create_item("cal", EVENT.to_vec())
                .unwrap_err();
            assert_eq!(code(&err), expected, "{status} {body}: {err:#}");
        }
    }

    #[test]
    fn a_duplicate_uid_and_a_stale_version_carry_their_code() {
        let conflict = precondition(CALDAV, "no-uid-conflict");
        let err = answering(409, &conflict)
            .create_item("cal", EVENT.to_vec())
            .unwrap_err();
        assert_eq!(code(&err), Some(ErrorCode::ItemDuplicate), "{err:#}");

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
