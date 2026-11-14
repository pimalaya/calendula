//! # Microsoft Graph backend
//!
//! The Microsoft Graph adapter for the shared cross-protocol client,
//! projecting [`io_msgraph`]'s calendars and events onto calendula's shared
//! types. A calendar is a Graph calendar and an item a lone event or a
//! series master, keeping the event id Graph returned, verbatim; the
//! `changeKey` serves as the etag.
//!
//! Graph narrows a listing by time only through the calendar view, which
//! expands every series into its occurrences, so a listing reads the
//! calendar's events whole and a [`CalendarTimeRange`] filters them
//! locally, as on vdir.
//!
//! A series is one item: its master read by id, the only read Graph
//! returns its cancelled occurrences on, and its exceptions read from the
//! instances of its own date range. Only the series master is written
//! back: an exception edited here does not push.
//!
//! [`CalendarTimeRange`]: crate::shared::item::CalendarTimeRange

use std::collections::BTreeMap;

use anyhow::{Context, Result, bail};
use ical::component::IcalComponentKind;
use io_msgraph::v1::{
    client::{MsgraphClientStd, MsgraphClientStdConnectOptions, MsgraphClientStdError},
    field::MsgraphField,
    rest::users::{
        calendars::{MsgraphCalendar, list::MsgraphCalendarsListParams},
        events::{
            MsgraphEvent, MsgraphEventType,
            ical::{MSGRAPH_EVENT_ICAL_SELECT, MSGRAPH_EVENT_STASH_EXPAND},
            list::MsgraphEventsListParams,
        },
    },
};
use jiff::{ToSpan, civil::Date, tz::TimeZone};
use log::warn;
use pimalaya_config::secret::SecretResolver;
use pimalaya_stream::proxy::Proxy;
use secrecy::ExposeSecret;

use pimalaya_cli::error::{CodedError, code_of};

use crate::{
    config::{MsgraphConfig, default_msgraph_alpn},
    error::{self, ErrorCode, recoded},
    shared::{
        calendar::{Calendar, CalendarDiff},
        client::window,
        item::{CalendarItem, CalendarItemList, CalendarItemQuery},
    },
};

/// How many events to ask for per page.
const PAGE_SIZE: u32 = 500;

/// How far past its start an open-ended series is searched for exceptions.
const OPEN_SERIES_YEARS: i16 = 5;

/// The shared-API glue over a connected Microsoft Graph client.
pub struct MsgraphBackend {
    client: MsgraphClientStd,
}

impl MsgraphBackend {
    /// Connects to Microsoft Graph with the configured bearer token.
    ///
    /// The token is resolved through `resolver`, so an account naming one
    /// credential command from several of its backends spawns it once.
    pub fn new(config: MsgraphConfig, resolver: &mut SecretResolver) -> Result<Self> {
        let token = resolver.resolve(config.auth.token.clone())?;
        let alpn = config.alpn.clone().unwrap_or_else(default_msgraph_alpn);
        let options = MsgraphClientStdConnectOptions {
            tls: config.tls.clone().into_tls(alpn),
            proxy: Proxy::None,
            user_id: config.user_id.clone(),
        };
        let client = MsgraphClientStd::connect(token.expose_secret(), options)
            .context("Cannot connect to Microsoft Graph")?;

        Ok(Self { client })
    }

    /// Lists every calendar of the user.
    pub fn list_calendars(&mut self) -> Result<Vec<Calendar>> {
        let params = MsgraphCalendarsListParams {
            top: Some(100),
            ..Default::default()
        };
        let mut page = self.client.calendars_list(&params)?.response;

        let mut calendars = Vec::new();
        loop {
            calendars.extend(page.value.into_iter().map(calendar_from));

            let Some(next) = page.next_link else {
                break;
            };
            page = self.client.page_get(&next, None)?.response;
        }

        Ok(calendars)
    }

    /// Creates a calendar, Graph minting its id, so the requested one cannot
    /// be honoured and the assigned one is returned instead.
    ///
    /// Graph keeps no description, and names its colours from a palette
    /// rather than taking any RGB, so neither is accepted.
    pub fn create_calendar(
        &mut self,
        id: &str,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<String> {
        if description.is_some() || color.is_some() {
            bail!("Microsoft Graph calendars carry neither a description nor an RGB colour");
        }

        let calendar = MsgraphCalendar {
            name: MsgraphField::Set(name.to_owned()),
            ..Default::default()
        };
        let created = self.client.calendar_create(&calendar)?.response;

        if created.id != id {
            warn!(
                "graph assigned the calendar id `{}`, not `{id}`",
                created.id
            );
        }

        Ok(created.id)
    }

    /// Renames a calendar, the only field of a patch Graph can hold.
    pub fn update_calendar(&mut self, id: &str, patch: CalendarDiff) -> Result<()> {
        if patch.description.is_some() || patch.color.is_some() {
            bail!("Microsoft Graph calendars carry neither a description nor an RGB colour");
        }

        if let Some(name) = patch.name {
            let calendar = MsgraphCalendar {
                name: MsgraphField::Set(name),
                ..Default::default()
            };
            self.client.calendar_update(id, &calendar)?;
        }

        Ok(())
    }

    /// Deletes a calendar and every event it holds; Graph refuses the
    /// default calendar.
    pub fn delete_calendar(&mut self, id: &str) -> Result<()> {
        self.client.calendar_delete(id)?;
        Ok(())
    }

    /// Lists the lone events and series masters of a calendar, projected
    /// onto iCalendar documents, a range filtering them locally.
    pub fn list_items(
        &mut self,
        calendar_id: &str,
        query: CalendarItemQuery<'_>,
    ) -> Result<CalendarItemList> {
        // NOTE: Graph models a VEVENT and nothing else, so any other kind
        // is answered without a round-trip.
        if query
            .kind
            .is_some_and(|kind| kind != IcalComponentKind::VEvent)
        {
            return Ok(CalendarItemList::default());
        }

        let params = MsgraphEventsListParams {
            top: Some(PAGE_SIZE),
            select: Some(MSGRAPH_EVENT_ICAL_SELECT),
            expand: Some(MSGRAPH_EVENT_STASH_EXPAND),
            ..Default::default()
        };
        let mut page = self
            .client
            .events_list(Some(calendar_id), &params)?
            .response;

        let mut events = Vec::new();
        loop {
            events.extend(page.value);

            let Some(next) = page.next_link else {
                break;
            };
            page = self.client.events_list_from_link(&next)?.response;
        }

        let mut items = Vec::new();
        let mut errors = Vec::new();
        for event in unique(events)? {
            let id = event.id.clone();
            let (event, exceptions) = match self.reread(event) {
                Ok(read) => read,
                // NOTE: an event gone since the listing is left out.
                Err(err) if code_of(&err) == Some(ErrorCode::ItemNotFound) => continue,
                Err(err) => {
                    let err = err.context(format!("Cannot read event `{id}` again"));
                    errors.push(error::unreadable(Some(&id), err));
                    continue;
                }
            };
            let item = item_from(calendar_id, &event, &exceptions);
            if item.starts_within(query.range) {
                items.push(item);
            }
        }

        // NOTE: the whole calendar is read for each window, so its errors
        // are named on the first alone.
        if query.skip > 0 {
            errors.clear();
        }

        Ok(CalendarItemList {
            items: window(items, query.skip, query.take),
            errors,
        })
    }

    /// A listed event with its exceptions, a series master read again by
    /// its id: a listing leaves out cancelledOccurrences, which Graph
    /// returns only on such a read.
    fn reread(&mut self, event: MsgraphEvent) -> Result<(MsgraphEvent, Vec<MsgraphEvent>)> {
        let event = match event.event_type {
            Some(MsgraphEventType::SeriesMaster) => self.event(&event.id)?,
            _ => event,
        };
        let exceptions = self.exceptions(&event)?;
        Ok((event, exceptions))
    }

    /// Reads one event, with the exceptions of a series folded into it.
    pub fn get_item(&mut self, calendar_id: &str, item_id: &str) -> Result<CalendarItem> {
        let event = self
            .event(item_id)
            .with_context(|| format!("Read item `{item_id}` from calendar `{calendar_id}`"))?;
        let exceptions = self.exceptions(&event)?;

        Ok(item_from(calendar_id, &event, &exceptions))
    }

    /// Creates an event from an iCalendar document, its UID riding the
    /// projection's stash.
    pub fn create_item(&mut self, calendar_id: &str, contents: Vec<u8>) -> Result<String> {
        let event = MsgraphEvent::create_from_ical(&contents)?;
        let created = self
            .client
            .event_create(Some(calendar_id), &event)?
            .response;

        Ok(created.id)
    }

    /// Replaces a series master from an iCalendar document, sending only
    /// what changed against the server copy, refused when its `changeKey`
    /// moved since `if_match`.
    pub fn update_item(
        &mut self,
        _calendar_id: &str,
        item_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<()> {
        let current = self.event(item_id)?;
        if let Some(expected) = if_match
            && current.change_key.as_deref() != Some(expected)
        {
            return Err(changed(item_id));
        }

        let exceptions = self.exceptions(&current)?;
        let exceptions: Vec<&MsgraphEvent> = exceptions.iter().collect();
        let base = current.to_ical_series(&exceptions);

        let patch = MsgraphEvent::update_from_ical(&contents, base.as_bytes())?;
        self.client
            .event_update(item_id, &patch)
            .map_err(item_refusal)?;

        Ok(())
    }

    /// Deletes an event, the whole series for a master.
    ///
    /// An `if_match` is checked against the event's `changeKey`, read
    /// first, as an update checks it.
    pub fn delete_item(
        &mut self,
        _calendar_id: &str,
        item_id: &str,
        if_match: Option<&str>,
    ) -> Result<()> {
        if let Some(expected) = if_match
            && self.event(item_id)?.change_key.as_deref() != Some(expected)
        {
            return Err(changed(item_id));
        }
        self.client.event_delete(item_id).map_err(item_refusal)?;
        Ok(())
    }

    /// Reads one event with everything the projection reads.
    fn event(&mut self, id: &str) -> Result<MsgraphEvent> {
        let event = self
            .client
            .event_get(
                id,
                Some(MSGRAPH_EVENT_ICAL_SELECT),
                Some(MSGRAPH_EVENT_STASH_EXPAND),
            )
            .map_err(item_refusal)?
            .response;
        Ok(event)
    }

    /// The exceptions of a series, read from the instances of its own date
    /// range; none for a lone event.
    fn exceptions(&mut self, event: &MsgraphEvent) -> Result<Vec<MsgraphEvent>> {
        if event.event_type != Some(MsgraphEventType::SeriesMaster) {
            return Ok(Vec::new());
        }

        let Some((start, end)) = series_window(event) else {
            warn!(
                "series `{}` has no readable range, its exceptions are skipped",
                event.id
            );
            return Ok(Vec::new());
        };

        let params = MsgraphEventsListParams {
            top: Some(PAGE_SIZE),
            // NOTE: an exception needs its originalStart for its
            // RECURRENCE-ID, which the default listing leaves out.
            select: Some(MSGRAPH_EVENT_ICAL_SELECT),
            ..Default::default()
        };
        let mut page = self
            .client
            .event_instances(&event.id, &start, &end, &params)
            .map_err(item_refusal)?
            .response;

        let mut exceptions = Vec::new();
        loop {
            exceptions.extend(
                page.value
                    .into_iter()
                    .filter(|instance| instance.event_type == Some(MsgraphEventType::Exception)),
            );

            let Some(next) = page.next_link else {
                break;
            };
            page = self.client.events_list_from_link(&next)?.response;
        }

        unique(exceptions)
    }
}

/// Projects a Graph calendar onto the shared [`Calendar`].
fn calendar_from(calendar: MsgraphCalendar) -> Calendar {
    Calendar {
        name: calendar
            .name
            .as_option()
            .cloned()
            .unwrap_or_else(|| calendar.id.clone()),
        id: calendar.id,
        description: None,
        color: calendar.hex_color,
        default: false,
    }
}

/// Projects an event and the exceptions of its series onto the shared
/// [`CalendarItem`], its contents synthesized by the projection.
fn item_from(calendar_id: &str, event: &MsgraphEvent, exceptions: &[MsgraphEvent]) -> CalendarItem {
    let exceptions: Vec<&MsgraphEvent> = exceptions.iter().collect();
    CalendarItem {
        id: event.id.clone(),
        calendar_id: calendar_id.to_owned(),
        etag: event.change_key.clone(),
        contents: event.to_ical_series(&exceptions).into_bytes(),
    }
}

/// Drops the events a page boundary repeated, Graph overlapping
/// consecutive `nextLink` pages; one id read at two revisions is an error.
fn unique(events: Vec<MsgraphEvent>) -> Result<Vec<MsgraphEvent>> {
    let mut seen = BTreeMap::new();
    let mut unique = Vec::with_capacity(events.len());

    for event in events {
        match seen.get(&event.id) {
            None => {
                seen.insert(event.id.clone(), event.change_key.clone());
                unique.push(event);
            }
            Some(change_key) if *change_key == event.change_key => {}
            Some(_) => bail!(
                "Event `{}` was listed twice at different revisions by Microsoft Graph",
                event.id
            ),
        }
    }

    Ok(unique)
}

/// The refusal of a write whose `--if-match` names a `changeKey` the event
/// no longer has, read before the write: Graph takes no `If-Match`.
fn changed(item_id: &str) -> anyhow::Error {
    let message = format!("Item `{item_id}` changed on Microsoft Graph since it was read");
    CodedError::new(ErrorCode::PreconditionFailed, message).into()
}

/// The code of a Graph refusal in an error chain: Graph's
/// `ErrorQuotaExceeded` or a 507 is `quota-exceeded`, a 429
/// `rate-limited`, a 403 `permission-denied`, a 412 `precondition-failed`.
pub(crate) fn refusal_code(cause: &(dyn std::error::Error + 'static)) -> Option<ErrorCode> {
    let MsgraphClientStdError::Send(send) = cause.downcast_ref::<MsgraphClientStdError>()? else {
        return None;
    };
    let err = send.api()?;

    match (err.status, err.code.as_str()) {
        (_, "ErrorQuotaExceeded") | (507, _) => Some(ErrorCode::QuotaExceeded),
        (429, _) => Some(ErrorCode::RateLimited),
        (403, _) => Some(ErrorCode::PermissionDenied),
        (412, _) => Some(ErrorCode::PreconditionFailed),
        _ => None,
    }
}

/// The failure of a request naming one event: a 404, or an id Graph reads
/// as no id of its own (400 `ErrorInvalidIdMalformed`), is
/// `item-not-found`.
fn item_refusal(err: MsgraphClientStdError) -> anyhow::Error {
    let missing = match &err {
        MsgraphClientStdError::Send(send) => send.api().is_some_and(|api| {
            api.status == 404 || (api.status == 400 && api.code == "ErrorInvalidIdMalformed")
        }),
        _ => false,
    };

    match missing {
        true => recoded(ErrorCode::ItemNotFound, err),
        false => err.into(),
    }
}

/// The window a series' exceptions fall in: its range, an open-ended one
/// capped past its start.
fn series_window(master: &MsgraphEvent) -> Option<(String, String)> {
    let (start, end) = master.recurrence.as_option()?.bounds()?;
    let end = match end {
        Some(end) => end,
        None => start.checked_add(OPEN_SERIES_YEARS.years()).ok()?,
    };

    let instant = |date: Date| -> Option<String> {
        let zoned = date.to_zoned(TimeZone::UTC).ok()?;
        Some(zoned.timestamp().strftime("%Y-%m-%dT%H:%M:%SZ").to_string())
    };

    Some((instant(start)?, instant(end.checked_add(1.day()).ok()?)?))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        io::{Read, Result as IoResult, Write},
    };

    use super::*;

    /// A stream answering each read with the next canned response.
    struct Scripted {
        responses: VecDeque<Vec<u8>>,
        written: Vec<u8>,
    }

    impl Read for Scripted {
        fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
            let response = self.responses.pop_front().unwrap_or_default();
            buf[..response.len()].copy_from_slice(&response);
            Ok(response.len())
        }
    }

    impl Write for Scripted {
        fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
            self.written.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> IoResult<()> {
            Ok(())
        }
    }

    #[test]
    fn a_calendar_page_link_off_the_graph_origin_is_not_followed() {
        let pages = [
            r#"{"value":[{"id":"A"}],"@odata.nextLink":"https://graph.example.org/v1.0/next"}"#,
            r#"{"value":[{"id":"B"}]}"#,
        ];
        let response = |body: &str| {
            let len = body.len();
            format!("HTTP/1.1 200 OK\r\nContent-Length: {len}\r\n\r\n{body}").into_bytes()
        };
        let stream = Scripted {
            responses: pages.into_iter().map(response).collect(),
            written: Vec::new(),
        };
        let mut backend = MsgraphBackend {
            client: MsgraphClientStd::new(stream, "made-up", Default::default()),
        };

        let err = backend.list_calendars().unwrap_err().to_string();
        assert!(err.contains("graph.example.org"), "{err}");

        let stream = backend.client.stream.as_any_mut();
        let written = &stream.downcast_mut::<Scripted>().unwrap().written;
        let written = String::from_utf8_lossy(written);
        assert!(!written.contains("graph.example.org"), "{written}");
    }

    fn event(id: &str, change_key: &str) -> MsgraphEvent {
        MsgraphEvent {
            id: id.into(),
            change_key: Some(change_key.into()),
            ..Default::default()
        }
    }

    #[test]
    fn an_event_repeated_across_pages_lists_once() {
        let events = vec![event("A", "1"), event("B", "1"), event("B", "1")];
        let ids: Vec<String> = unique(events)
            .unwrap()
            .into_iter()
            .map(|event| event.id)
            .collect();

        assert_eq!(ids, ["A", "B"]);
    }

    #[test]
    fn an_event_repeated_at_another_revision_is_refused() {
        assert!(unique(vec![event("A", "1"), event("A", "2")]).is_err());
    }
}

#[cfg(test)]
pub(crate) mod scripted {
    use std::{
        collections::VecDeque,
        io::{Read, Result as IoResult, Write},
    };

    use super::*;

    /// A stream answering each read with the next canned response, keeping
    /// the requests written to it.
    pub(crate) struct Answers {
        responses: VecDeque<Vec<u8>>,
        pub(crate) written: Vec<u8>,
    }

    impl Read for Answers {
        fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
            let response = self.responses.pop_front().unwrap_or_default();
            buf[..response.len()].copy_from_slice(&response);
            Ok(response.len())
        }
    }

    impl Write for Answers {
        fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
            self.written.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> IoResult<()> {
            Ok(())
        }
    }

    /// A backend whose server answers each `(status, body)` in turn.
    pub(crate) fn answering(answers: &[(u16, &str)]) -> MsgraphBackend {
        let response = |(status, body): &(u16, &str)| {
            let len = body.len();
            format!(
                "HTTP/1.1 {status} Answer\r\nContent-Type: application/json\r\n\
                 Content-Length: {len}\r\n\r\n{body}"
            )
            .into_bytes()
        };
        let stream = Answers {
            responses: answers.iter().map(response).collect(),
            written: Vec::new(),
        };

        MsgraphBackend {
            client: MsgraphClientStd::new(stream, "made-up", Default::default()),
        }
    }

    /// The requests `backend` sent, one request line each.
    pub(crate) fn requests(backend: &mut MsgraphBackend) -> Vec<String> {
        let stream = backend.client.stream.as_any_mut();
        let written = &stream.downcast_mut::<Answers>().unwrap().written;
        String::from_utf8_lossy(written)
            .lines()
            .filter(|line| line.ends_with(" HTTP/1.1"))
            .map(str::to_owned)
            .collect()
    }

    /// A Graph error envelope.
    pub(crate) fn refusal(code: &str) -> String {
        format!(r#"{{"error":{{"code":"{code}","message":"Refused"}}}}"#)
    }

    /// An event as a read by id answers it.
    const EVENT: &str = r#"{"id":"E1","changeKey":"k1","subject":"Standup"}"#;

    #[test]
    fn a_refusal_carries_its_code() {
        let cases = [
            (403, "ErrorAccessDenied", ErrorCode::PermissionDenied),
            (507, "ErrorQuotaExceeded", ErrorCode::QuotaExceeded),
            (400, "ErrorQuotaExceeded", ErrorCode::QuotaExceeded),
            (429, "ApplicationThrottled", ErrorCode::RateLimited),
            (
                412,
                "ErrorIrresolvableConflict",
                ErrorCode::PreconditionFailed,
            ),
        ];

        for (status, code, expected) in cases {
            let body = refusal(code);
            let mut backend = answering(&[(status, &body)]);
            let err = backend.list_calendars().unwrap_err();
            assert_eq!(crate::error::code(&err), Some(expected), "{status} {err:#}");
        }

        let body = refusal("ErrorInternalServerError");
        let err = answering(&[(500, &body)]).list_calendars().unwrap_err();
        assert_eq!(crate::error::code(&err), None, "{err:#}");
    }

    #[test]
    fn an_event_graph_does_not_hold_is_not_found() {
        let missing = refusal("ErrorItemNotFound");
        let malformed = refusal("ErrorInvalidIdMalformed");

        for (status, body) in [(404, &missing), (400, &malformed)] {
            let err = answering(&[(status, body)])
                .get_item("cal", "nope")
                .unwrap_err();
            assert_eq!(
                crate::error::code(&err),
                Some(ErrorCode::ItemNotFound),
                "{err:#}"
            );
        }

        let other = refusal("ErrorInvalidRequest");
        let err = answering(&[(400, &other)])
            .get_item("cal", "nope")
            .unwrap_err();
        assert_eq!(crate::error::code(&err), None, "{err:#}");
    }

    /// A series master the server refuses to read again is an entry coded
    /// by the refusal, the other events listed, and one gone since the
    /// listing (404) is left out with none; a later window names neither.
    #[test]
    fn a_refused_reread_is_an_entry_and_a_gone_one_is_left_out() {
        let listing = r#"{"value":[
            {"id":"S1","changeKey":"k1","subject":"Weekly","type":"seriesMaster"},
            {"id":"S2","changeKey":"k1","subject":"Gone","type":"seriesMaster"},
            {"id":"L","changeKey":"k1","subject":"Lone","type":"singleInstance"}
        ]}"#;
        let denied = refusal("ErrorAccessDenied");
        let missing = refusal("ErrorItemNotFound");
        let answers = [(200, listing), (403, &denied), (404, &missing)];

        let listed = answering(&answers)
            .list_items("cal", CalendarItemQuery::default())
            .unwrap();
        let ids: Vec<&str> = listed.items.iter().map(|item| item.id.as_str()).collect();
        assert_eq!(ids, ["L"]);

        let [entry] = &listed.errors[..] else {
            panic!("{:?}", listed.errors);
        };
        assert_eq!(entry.id.as_deref(), Some("S1"));
        assert_eq!(entry.code, Some(ErrorCode::PermissionDenied));
        assert_eq!(entry.error, "Cannot read event `S1` again");
        assert_eq!(
            entry.sources,
            ["Microsoft Graph API returned HTTP 403 (ErrorAccessDenied): Refused"]
        );

        let later = CalendarItemQuery {
            skip: 1,
            ..Default::default()
        };
        let listed = answering(&answers).list_items("cal", later).unwrap();
        assert!(listed.errors.is_empty(), "{:?}", listed.errors);
    }

    /// A write the server refuses prints its code in the failure object,
    /// its text and its sources as before.
    #[test]
    fn a_refused_write_prints_its_code_in_the_failure_object() {
        use pimalaya_cli::error::ErrorReport;

        let denied = refusal("ErrorAccessDenied");
        let err = answering(&[(403, &denied)])
            .delete_item("cal", "E1", None)
            .unwrap_err()
            .context("Delete event `E1`");
        let text = format!("{err:#}");

        let err = crate::error::with_code(err);
        assert_eq!(format!("{err:#}"), text);

        let json = serde_json::to_value(ErrorReport::<ErrorCode>::from(err)).unwrap();
        assert_eq!(json["code"], "permission-denied", "{json}");
        assert_eq!(json["error"], "Delete event `E1`", "{json}");
        assert_eq!(
            json["sources"],
            serde_json::json!([
                "Microsoft Graph API returned HTTP 403 (ErrorAccessDenied): Refused"
            ]),
            "{json}"
        );
    }

    /// A version that moved since `--if-match` read it refuses the write
    /// before it is sent, coded as a 412 would be.
    #[test]
    fn a_moved_change_key_refuses_the_delete_as_a_precondition() {
        let mut backend = answering(&[(200, EVENT)]);
        let err = backend.delete_item("cal", "E1", Some("k0")).unwrap_err();

        assert_eq!(
            pimalaya_cli::error::code_of(&err),
            Some(ErrorCode::PreconditionFailed)
        );
        assert_eq!(requests(&mut backend).len(), 1, "no DELETE sent");
    }
}
