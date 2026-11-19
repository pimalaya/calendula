//! # JMAP backend
//!
//! The JMAP for Calendars adapter (draft-ietf-jmap-calendars) of the
//! shared cross-protocol client. A calendar is a JMAP Calendar, its name,
//! description and colour mapped both ways, and an item one CalendarEvent,
//! keyed by its id verbatim, converted through [`project`].
//!
//! A recurring event carries its overrides in itself, so a series is one
//! item with nothing to fold. A range reaches the server as the `after`
//! and `before` of `CalendarEvent/query`, recurrences unexpanded, and a
//! page as its `position` and `limit`.
//!
//! JMAP keeps no per-event entity tag, so `--if-match` is compared with
//! the hash of the event read just before the write. Each read checks the
//! event's `calendarIds`, so a read, an update or a delete acts only on an
//! event the named calendar holds.

use std::collections::BTreeMap;

use anyhow::{Result, anyhow};
use ical::component::IcalComponentKind;
use io_jmap::{
    calendars::{
        JMAP_CALENDARS_CAPABILITY,
        calendar::{
            JmapCalendar,
            get::JmapCalendarGetOptions,
            set::{JmapCalendarCreate, JmapCalendarSetArgs, JmapCalendarUpdate},
        },
        calendar_event::{
            JmapCalendarEvent,
            get::JmapCalendarEventGetOptions,
            query::{
                JmapCalendarEventFilter, JmapCalendarEventQueryOptions,
                JmapCalendarEventSortComparator, JmapCalendarEventSortProperty,
            },
            set::{JmapCalendarEventSetArgs, JmapCalendarEventSetItemError},
        },
    },
    client::JmapClientStd,
};
use log::warn;
use pimalaya_cli::error::{CodedError, with_code};
use pimalaya_config::secret::SecretResolver;

use crate::{
    config::JmapConfig,
    error::{self, ErrorCode},
    jmap::{
        client::connect,
        error::{JmapSetError, set_error},
        project,
    },
    shared::{
        calendar::{Calendar, CalendarDiff},
        item::{CalendarItemList, CalendarItemQuery, CalendarTimeRange},
    },
};

/// The client id a create names its one event by.
const CREATE_ID: &str = "c0";

/// The shared-API glue over a JMAP client with an open session.
pub struct JmapBackend {
    client: JmapClientStd,
}

impl JmapBackend {
    /// Connects and opens the session, refused when it serves no
    /// calendars.
    pub fn new(config: JmapConfig, resolver: &mut SecretResolver) -> Result<Self> {
        Ok(Self {
            client: connect(config, resolver)?,
        })
    }

    /// Lists the account's calendars.
    pub fn list_calendars(&mut self) -> Result<Vec<Calendar>> {
        let out = self.client.calendar_get(Default::default())?;
        Ok(out.calendars.into_iter().map(calendar_from).collect())
    }

    /// Creates a calendar, the server minting its id, which is returned;
    /// refused before any request when the account may not create one.
    pub fn create_calendar(
        &mut self,
        id: Option<&str>,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<String> {
        if self.may_create_calendar() == Some(false) {
            let message = "The JMAP account may not create calendars (`mayCreateCalendar`)";
            return Err(CodedError::new(ErrorCode::PermissionDenied, message).into());
        }

        let calendar = JmapCalendarCreate {
            name: Some(name.to_owned()),
            description: description.map(ToOwned::to_owned),
            color: color.map(ToOwned::to_owned),
            ..Default::default()
        };
        let args = JmapCalendarSetArgs {
            create: Some(BTreeMap::from([(CREATE_ID.to_owned(), calendar)])),
            ..Default::default()
        };
        let mut out = self.client.calendar_set(args)?;

        if let Some(err) = out.not_created.remove(CREATE_ID) {
            return Err(set_error("JMAP Calendar create rejected", &err));
        }

        let created = out
            .created
            .remove(CREATE_ID)
            .and_then(|created| created.id)
            .ok_or_else(|| anyhow!("JMAP Calendar/set answered no id for the calendar created"))?;

        if let Some(id) = id.filter(|id| *id != created) {
            warn!("the JMAP server assigned the calendar id `{created}`, not `{id}`");
        }

        Ok(created)
    }

    /// Whether the account may create calendars, as its calendars
    /// capability states, `None` when it states nothing.
    fn may_create_calendar(&self) -> Option<bool> {
        let session = self.client.session()?;
        let account = session.account_for(JMAP_CALENDARS_CAPABILITY).ok()?;
        let capabilities = &session.accounts.get(account)?.account_capabilities;
        let calendars = capabilities.get(JMAP_CALENDARS_CAPABILITY)?;
        calendars.get("mayCreateCalendar")?.as_bool()
    }

    /// Applies `patch` to the calendar `id`, a cleared field sent as
    /// `null`.
    pub fn update_calendar(&mut self, id: &str, patch: CalendarDiff) -> Result<()> {
        if patch.name.is_none() && patch.description.is_none() && patch.color.is_none() {
            return Ok(());
        }

        let update = JmapCalendarUpdate {
            name: patch.name,
            description: patch.description,
            color: patch.color,
            ..Default::default()
        };
        let args = JmapCalendarSetArgs {
            update: Some(BTreeMap::from([(id.to_owned(), update)])),
            ..Default::default()
        };
        let mut out = self.client.calendar_set(args)?;

        match out.not_updated.remove(id) {
            Some(err) => Err(set_error("JMAP Calendar update rejected", &err)),
            None => Ok(()),
        }
    }

    /// Deletes the calendar `id` with every event it holds
    /// (`onDestroyRemoveEvents`).
    pub fn delete_calendar(&mut self, id: &str) -> Result<()> {
        let args = JmapCalendarSetArgs {
            destroy: Some(vec![id.to_owned()]),
            on_destroy_remove_events: Some(true),
            ..Default::default()
        };
        let mut out = self.client.calendar_set(args)?;

        match out.not_destroyed.remove(id) {
            Some(err) => Err(set_error("JMAP Calendar destroy rejected", &err)),
            None => Ok(()),
        }
    }

    /// Lists the window `query` names of the events of `calendar_id`, one
    /// item per event, a series with its overrides.
    ///
    /// A to-do or a journal listing is answered empty without a round
    /// trip. A query that fails or finds nothing is checked against the
    /// calendars, so an unknown calendar fails `calendar-not-found`
    /// rather than reads as empty.
    pub fn list_items(
        &mut self,
        calendar_id: &str,
        query: CalendarItemQuery<'_>,
    ) -> Result<CalendarItemList> {
        if query
            .kind
            .is_some_and(|kind| kind != IcalComponentKind::VEvent)
        {
            return Ok(CalendarItemList::default());
        }

        let filter = filter(calendar_id, query.range)?;

        let events = match self.query(filter, query.skip, query.take) {
            Ok(events) if !events.is_empty() => events,
            queried => match (self.holds_calendar(calendar_id)?, queried) {
                (true, queried) => queried?,
                (false, Ok(_)) => {
                    let message = format!("Calendar `{calendar_id}` not found");
                    return Err(error::calendar_not_found(message));
                }
                (false, Err(err)) => {
                    let message = format!("Calendar `{calendar_id}` not found");
                    let err = with_code(err, |_| Some(ErrorCode::CalendarNotFound));
                    return Err(err.context(message));
                }
            },
        };

        let mut items = Vec::new();
        let mut errors = Vec::new();

        for event in events {
            match project::to_item(calendar_id, &event) {
                Ok(item) => items.push(item),
                Err(err) => {
                    let err = err.context("Cannot convert the JMAP event to iCalendar");
                    errors.push(error::unreadable(event.id.as_deref(), err));
                }
            }
        }

        Ok(CalendarItemList {
            items,
            errors,
            places: None,
        })
    }

    /// The events `filter` matches, the `take` after the first `skip`, in
    /// start order, pages followed while the server answers fewer than
    /// asked.
    fn query(
        &mut self,
        filter: JmapCalendarEventFilter,
        skip: usize,
        take: Option<usize>,
    ) -> Result<Vec<JmapCalendarEvent>> {
        let mut events = Vec::new();

        loop {
            let limit = take.map(|take| take.saturating_sub(events.len()));
            if limit == Some(0) {
                break;
            }

            let position = skip.saturating_add(events.len());
            let opts = JmapCalendarEventQueryOptions {
                filter: Some(filter.clone()),
                sort: Some(vec![
                    sort(JmapCalendarEventSortProperty::Start),
                    sort(JmapCalendarEventSortProperty::Uid),
                ]),
                position: Some(position as u64),
                limit: limit.map(|limit| limit as u64),
                ..Default::default()
            };
            let out = self.client.calendar_event_query(opts)?;
            let answered = out.events.len() + out.not_found.len();
            events.extend(out.events);

            let ended = out
                .total
                .is_none_or(|total| (position + answered) as u64 >= total);
            if answered == 0 || ended {
                break;
            }
        }

        Ok(events)
    }

    /// Whether the account holds the calendar `id`.
    fn holds_calendar(&mut self, id: &str) -> Result<bool> {
        let opts = JmapCalendarGetOptions {
            ids: Some(vec![id.to_owned()]),
            ..Default::default()
        };
        let out = self.client.calendar_get(opts)?;

        Ok(out
            .calendars
            .iter()
            .any(|calendar| calendar.id.as_deref() == Some(id)))
    }

    /// Reads the event `item_id` of `calendar_id`.
    pub fn get_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
    ) -> Result<crate::shared::item::CalendarItem> {
        let event = self
            .held(calendar_id, item_id, None)
            .map_err(|err| error::read_failed(err, item_id, calendar_id))?;

        project::to_item(calendar_id, &event)
            .map_err(|err| error::read_failed(err, item_id, calendar_id))
    }

    /// The event `item_id`, its `properties` alone when given, failing
    /// `item-not-found` unless the calendar `calendar_id` holds it per its
    /// `calendarIds`.
    fn held(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        properties: Option<Vec<String>>,
    ) -> Result<JmapCalendarEvent> {
        let opts = JmapCalendarEventGetOptions {
            ids: Some(vec![item_id.to_owned()]),
            properties,
            ..Default::default()
        };
        let out = self.client.calendar_event_get(opts)?;

        let missing = || {
            error::item_not_found(format!(
                "Item `{item_id}` not found in calendar `{calendar_id}`"
            ))
        };

        let event = out.events.into_iter().next().ok_or_else(missing)?;

        if event.calendar_ids.get(calendar_id) != Some(&true) {
            return Err(missing());
        }

        Ok(event)
    }

    /// Creates an event in `calendar_id` from an iCalendar document,
    /// returning the id the server minted.
    pub fn create_item(&mut self, calendar_id: &str, contents: Vec<u8>) -> Result<String> {
        let mut event = project::to_event(&contents)?;
        event.calendar_ids = BTreeMap::from([(calendar_id.to_owned(), true)]);

        let args = JmapCalendarEventSetArgs {
            create: Some(BTreeMap::from([(CREATE_ID.to_owned(), event)])),
            ..Default::default()
        };
        let mut out = self.client.calendar_event_set(args)?;

        if let Some(err) = out.not_created.remove(CREATE_ID) {
            return Err(set_error("JMAP CalendarEvent create rejected", &err));
        }

        out.created
            .remove(CREATE_ID)
            .and_then(|created| created.id)
            .ok_or_else(|| anyhow!("JMAP CalendarEvent/set answered no id for the event created"))
    }

    /// Updates the event `item_id` from an iCalendar document, sending a
    /// patch against the copy read now, refused when `if_match` is given
    /// and differs from that copy's version.
    pub fn update_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<()> {
        let staged = project::to_event(&contents)?;
        let base = self.held(calendar_id, item_id, None)?;

        if let Some(expected) = if_match
            && project::etag(&base) != expected
        {
            return Err(changed(item_id));
        }

        let patch = project::to_patch(&staged, &base);

        if patch.0.is_empty() {
            return Ok(());
        }

        let args = JmapCalendarEventSetArgs {
            update: Some(BTreeMap::from([(item_id.to_owned(), patch)])),
            ..Default::default()
        };
        let mut out = self.client.calendar_event_set(args)?;

        if let Some(err) = out.not_updated.remove(item_id) {
            return Err(item_refusal("update", calendar_id, item_id, &err));
        }

        Ok(())
    }

    /// Destroys the event `item_id`, a whole series for a recurring one.
    ///
    /// The event is read first, its `calendarIds` alone without an
    /// `if_match`, whole with one, compared with its version as an update
    /// compares it.
    pub fn delete_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        if_match: Option<&str>,
    ) -> Result<()> {
        match if_match {
            Some(expected) => {
                let current = self.held(calendar_id, item_id, None)?;

                if project::etag(&current) != expected {
                    return Err(changed(item_id));
                }
            }
            None => {
                let properties = vec![String::from("calendarIds")];
                self.held(calendar_id, item_id, Some(properties))?;
            }
        }

        let args = JmapCalendarEventSetArgs {
            destroy: Some(vec![item_id.to_owned()]),
            ..Default::default()
        };
        let mut out = self.client.calendar_event_set(args)?;

        if let Some(err) = out.not_destroyed.remove(item_id) {
            return Err(item_refusal("destroy", calendar_id, item_id, &err));
        }

        Ok(())
    }
}

/// The `CalendarEvent/query` filter of the events of `calendar_id` within
/// `range`, its bounds as the date-times the query reads.
///
/// io-jmap documents `after` and `before` as local date-times, read in the
/// query's `timeZone`, which calendula leaves to the server's default,
/// `Etc/UTC`: the UTC bounds of the range go as they are, without their
/// `Z`.
fn filter(calendar_id: &str, range: Option<&CalendarTimeRange>) -> Result<JmapCalendarEventFilter> {
    let bound = |stamp: Option<&String>| stamp.map(|stamp| date_time(stamp)).transpose();

    Ok(JmapCalendarEventFilter {
        in_calendar: Some(calendar_id.to_owned()),
        after: bound(range.and_then(|range| range.start.as_ref()))?,
        before: bound(range.and_then(|range| range.end.as_ref()))?,
        ..Default::default()
    })
}

/// The `YYYY-MM-DDTHH:MM:SS` date-time of an iCalendar UTC stamp.
fn date_time(stamp: &str) -> Result<String> {
    let civil = stamp
        .strip_suffix('Z')
        .ok_or_else(|| anyhow!("Cannot read the range bound `{stamp}`: it is not in UTC"))?;
    let date_time = jiff::civil::DateTime::strptime("%Y%m%dT%H%M%S", civil)
        .map_err(|err| anyhow!("Cannot read the range bound `{stamp}`: {err}"))?;

    Ok(date_time.strftime("%Y-%m-%dT%H:%M:%S").to_string())
}

/// An ascending sort on `property`.
fn sort(property: JmapCalendarEventSortProperty) -> JmapCalendarEventSortComparator {
    JmapCalendarEventSortComparator {
        property,
        is_ascending: Some(true),
    }
}

/// Projects a JMAP Calendar onto the shared [`Calendar`], its id standing
/// for a missing name.
fn calendar_from(calendar: JmapCalendar) -> Calendar {
    let id = calendar.id.unwrap_or_default();
    let name = calendar.name.filter(|name| !name.is_empty());

    Calendar {
        name: name.unwrap_or_else(|| id.clone()),
        id,
        description: calendar.description.filter(|text| !text.is_empty()),
        color: calendar.color.filter(|color| !color.is_empty()),
        default: calendar.is_default,
    }
}

/// The refusal of a write `verb` on the event `item_id`: a `notFound` is
/// the item missing from the calendar, the server's answer kept under it.
fn item_refusal(
    verb: &str,
    calendar_id: &str,
    item_id: &str,
    err: &JmapCalendarEventSetItemError,
) -> anyhow::Error {
    let refusal = set_error(&format!("JMAP CalendarEvent {verb} rejected"), err);

    match err.code() {
        Some(ErrorCode::ItemNotFound) => refusal.context(format!(
            "Item `{item_id}` not found in calendar `{calendar_id}`"
        )),
        _ => refusal,
    }
}

/// The refusal of a write whose `--if-match` names a version the event no
/// longer has, read before the write: JMAP takes no per-event `If-Match`.
fn changed(item_id: &str) -> anyhow::Error {
    let message = format!("Item `{item_id}` changed on the JMAP server since it was read");
    CodedError::new(ErrorCode::PreconditionFailed, message).into()
}

#[cfg(test)]
mod tests {
    use pimalaya_cli::error::code_of;
    use serde_json::{Value, json};

    use super::*;
    use crate::script::{Requests, Script, response};

    /// The backend over a JMAP server answering each request with the next
    /// of `bodies`, its session naming account `a1` for calendars, and the
    /// requests it receives.
    fn scripted(bodies: &[&str]) -> (JmapBackend, Requests) {
        scripted_as(bodies, json!({"mayCreateCalendar": true}))
    }

    /// As [`scripted`], the account's calendars capability `capability`.
    fn scripted_as(bodies: &[&str], capability: Value) -> (JmapBackend, Requests) {
        use io_jmap::rfc8620::session::JmapSession;
        use secrecy::SecretString;

        let session: JmapSession = serde_json::from_value(json!({
            "username": "", "capabilities": {JMAP_CALENDARS_CAPABILITY: {}},
            "accounts": {"a1": {"name": "a", "isPersonal": true, "isReadOnly": false,
                "accountCapabilities": {JMAP_CALENDARS_CAPABILITY: capability}}},
            "primaryAccounts": {JMAP_CALENDARS_CAPABILITY: "a1"},
            "apiUrl": "https://jmap.example.org/api/", "downloadUrl": "",
            "uploadUrl": "", "eventSourceUrl": "", "state": "",
        }))
        .unwrap();
        let responses = bodies
            .iter()
            .map(|body| response(200, "application/json", body));
        let (script, requests) = Script::new(responses);
        let auth = SecretString::from("Bearer token");
        let client = JmapClientStd::from_parts(script, auth, session);

        (JmapBackend { client }, requests)
    }

    /// The method calls of each request `requests` holds, as sent.
    fn calls(requests: &Requests) -> Vec<Value> {
        let requests = requests.lock().unwrap();
        requests
            .iter()
            .map(|request| {
                let request = String::from_utf8_lossy(request);
                let (_, body) = request.split_once("\r\n\r\n").unwrap();
                let body: Value = serde_json::from_str(body).unwrap();
                body["methodCalls"][0].clone()
            })
            .collect()
    }

    /// A method response of `name` carrying `args`.
    fn answer(name: &str, args: Value) -> String {
        json!({"methodResponses": [[name, args, "c0"]], "sessionState": "s"}).to_string()
    }

    /// A `CalendarEvent/get` answering `E1` in calendar `C1`.
    fn get() -> String {
        answer(
            "CalendarEvent/get",
            json!({"accountId": "a1", "state": "s1", "notFound": [], "list": [{
                "id": "E1", "calendarIds": {"C1": true}, "@type": "Event",
                "uid": "u1", "title": "Standup", "start": "2026-10-01T09:00:00",
                "duration": "PT30M", "updated": "2026-10-01T00:00:00Z",
            }]}),
        )
    }

    /// A `CalendarEvent/set` accepting every write.
    fn set() -> String {
        answer(
            "CalendarEvent/set",
            json!({"accountId": "a1", "oldState": "s1", "newState": "s2",
                "created": {"c0": {"id": "E9"}}, "updated": {"E1": null}, "destroyed": ["E1"]}),
        )
    }

    const EVENT: &[u8] = b"BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//t//EN\r\n\
        BEGIN:VEVENT\r\nUID:u1\r\nDTSTAMP:20261001T000000Z\r\n\
        DTSTART:20261001T090000\r\nDURATION:PT30M\r\nSUMMARY:Daily\r\n\
        END:VEVENT\r\nEND:VCALENDAR\r\n";

    const TODO: &[u8] = b"BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//t//EN\r\n\
        BEGIN:VTODO\r\nUID:t1\r\nDTSTAMP:20261001T000000Z\r\nEND:VTODO\r\nEND:VCALENDAR\r\n";

    /// `--if-match` is compared with the hash of the event the write reads:
    /// a stale one is refused `precondition-failed` before any set, the
    /// one a read reports lets the write through as a patch of what
    /// changed.
    #[test]
    fn a_write_is_guarded_by_the_hash_of_the_event_it_reads() {
        let (mut backend, _) = scripted(&[&get()]);
        let etag = backend.get_item("C1", "E1").unwrap().etag.unwrap();

        let stale = Some(ErrorCode::PreconditionFailed);

        let (mut backend, requests) = scripted(&[&get()]);
        let err = backend
            .update_item("C1", "E1", EVENT.to_vec(), Some("0"))
            .unwrap_err();
        assert_eq!(code_of(&err), stale, "{err:#}");
        assert_eq!(requests.lock().unwrap().len(), 1);

        let (mut backend, requests) = scripted(&[&get(), &set()]);
        backend
            .update_item("C1", "E1", EVENT.to_vec(), Some(&etag))
            .unwrap();
        let sent = calls(&requests);
        assert_eq!(sent[1][0], "CalendarEvent/set");
        assert_eq!(sent[1][1]["update"]["E1"], json!({"title": "Daily"}));

        let (mut backend, requests) = scripted(&[&get()]);
        let err = backend.delete_item("C1", "E1", Some("0")).unwrap_err();
        assert_eq!(code_of(&err), stale, "{err:#}");
        assert_eq!(requests.lock().unwrap().len(), 1);

        let (mut backend, requests) = scripted(&[&get(), &set()]);
        backend.delete_item("C1", "E1", Some(&etag)).unwrap();
        assert_eq!(calls(&requests)[1][1]["destroy"], json!(["E1"]));
    }

    /// An event of another calendar is not found in the one named: no
    /// read prints it there, no write reaches it.
    #[test]
    fn an_event_of_another_calendar_is_not_found() {
        let missing = Some(ErrorCode::ItemNotFound);

        let (mut backend, _) = scripted(&[&get()]);
        let err = backend.get_item("C2", "E1").unwrap_err();
        assert_eq!(code_of(&err), missing, "{err:#}");
        assert_eq!(err.to_string(), "Item `E1` not found in calendar `C2`");

        let (mut backend, requests) = scripted(&[&get(), &set()]);
        let err = backend
            .update_item("C2", "E1", EVENT.to_vec(), None)
            .unwrap_err();
        assert_eq!(code_of(&err), missing, "{err:#}");
        assert_eq!(requests.lock().unwrap().len(), 1);

        let (mut backend, requests) = scripted(&[&get(), &set()]);
        let err = backend.delete_item("C2", "E1", None).unwrap_err();
        assert_eq!(code_of(&err), missing, "{err:#}");
        let sent = calls(&requests);
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0][1]["properties"], json!(["calendarIds"]));
    }

    /// A to-do or a journal listing costs no request, and a write carrying
    /// no VEVENT is refused by what it carries before any request.
    #[test]
    fn a_to_do_is_neither_listed_nor_written() {
        let (mut backend, requests) = scripted(&[]);

        for kind in [IcalComponentKind::VTodo, IcalComponentKind::VJournal] {
            let query = CalendarItemQuery {
                kind: Some(kind),
                ..Default::default()
            };
            let listed = backend.list_items("C1", query).unwrap();
            assert!(listed.items.is_empty() && listed.errors.is_empty());
        }

        let err = backend.create_item("C1", TODO.to_vec()).unwrap_err();
        assert!(
            err.to_string().contains("carry no VEVENT but VTODO"),
            "{err}"
        );
        let err = backend
            .update_item("C1", "E1", TODO.to_vec(), None)
            .unwrap_err();
        assert!(
            err.to_string().contains("carry no VEVENT but VTODO"),
            "{err}"
        );

        assert!(requests.lock().unwrap().is_empty());
    }

    /// A create sends the event into the calendar named, envelope aside,
    /// and reports the id the server minted.
    #[test]
    fn a_create_reports_the_id_the_server_minted() {
        let (mut backend, requests) = scripted(&[&set()]);
        let id = backend.create_item("C1", EVENT.to_vec()).unwrap();
        assert_eq!(id, "E9");

        let sent = calls(&requests);
        let event = &sent[0][1]["create"]["c0"];
        assert_eq!(event["calendarIds"], json!({"C1": true}));
        assert_eq!(event["uid"], "u1");
        assert_eq!(event["title"], "Daily");
        assert!(event.get("id").is_none(), "{event}");
    }

    /// A range reaches the query as its `after` and `before`, recurrences
    /// unexpanded, a page as its `position` and `limit`, in start order;
    /// a page the server answers short is followed.
    #[test]
    fn a_window_is_pushed_down_to_the_query() {
        let page = |ids: &[&str], position: u64, total: u64| {
            let list: Vec<Value> = ids
                .iter()
                .map(|id| {
                    json!({"id": id, "calendarIds": {"C1": true}, "@type": "Event",
                    "uid": id, "start": "2026-11-16T09:00:00", "title": id})
                })
                .collect();
            json!({"methodResponses": [
                ["CalendarEvent/query", {"accountId": "a1", "queryState": "q",
                    "ids": ids, "position": position, "total": total,
                    "canCalculateChanges": false}, "c0"],
                ["CalendarEvent/get", {"accountId": "a1", "state": "s",
                    "list": list, "notFound": []}, "c1"],
            ], "sessionState": "s"})
            .to_string()
        };
        let (mut backend, requests) = scripted(&[&page(&["E3"], 2, 9), &page(&["E4"], 3, 9)]);

        let range = CalendarTimeRange {
            start: Some("20261116T000000Z".into()),
            end: Some("20261123T000000Z".into()),
        };
        let query = CalendarItemQuery {
            skip: 2,
            take: Some(2),
            range: Some(&range),
            kind: Some(IcalComponentKind::VEvent),
            ..Default::default()
        };
        let listed = backend.list_items("C1", query).unwrap();

        let ids: Vec<&str> = listed.items.iter().map(|item| item.id.as_str()).collect();
        assert_eq!(ids, ["E3", "E4"]);

        let sent = calls(&requests);
        assert_eq!(sent.len(), 2);
        let first = &sent[0][1];
        assert_eq!(sent[0][0], "CalendarEvent/query");
        assert_eq!(
            first["filter"],
            json!({"inCalendar": "C1", "after": "2026-11-16T00:00:00",
                "before": "2026-11-23T00:00:00"})
        );
        assert!(first.get("expandRecurrences").is_none(), "{first}");
        assert_eq!(first["position"], 2);
        assert_eq!(first["limit"], 2);
        assert_eq!(first["sort"][0]["property"], "start");
        assert_eq!(sent[1][1]["position"], 3);
        assert_eq!(sent[1][1]["limit"], 1);
    }

    /// A calendar the account does not hold fails `calendar-not-found`
    /// rather than lists empty, one it holds lists empty.
    #[test]
    fn an_unknown_calendar_is_not_found() {
        let empty = json!({"methodResponses": [
            ["CalendarEvent/query", {"accountId": "a1", "queryState": "q", "ids": [],
                "position": 0, "total": 0, "canCalculateChanges": false}, "c0"],
            ["CalendarEvent/get", {"accountId": "a1", "state": "s", "list": [],
                "notFound": []}, "c1"],
        ], "sessionState": "s"})
        .to_string();
        let calendars = |list: Value| {
            answer(
                "Calendar/get",
                json!({"accountId": "a1", "state": "s", "list": list, "notFound": []}),
            )
        };

        let (mut backend, _) = scripted(&[&empty, &calendars(json!([]))]);
        let err = backend
            .list_items("nope", CalendarItemQuery::default())
            .unwrap_err();
        assert_eq!(code_of(&err), Some(ErrorCode::CalendarNotFound), "{err:#}");

        let held = calendars(json!([{"id": "C1", "name": "Work"}]));
        let (mut backend, _) = scripted(&[&empty, &held]);
        let listed = backend
            .list_items("C1", CalendarItemQuery::default())
            .unwrap();
        assert!(listed.items.is_empty() && listed.errors.is_empty());
    }

    /// The calendar writes go out as `Calendar/set` calls of the calendar
    /// account: a create naming what it was given, an update nulling what
    /// it clears, a destroy removing the events with the calendar.
    #[test]
    fn the_calendar_writes_are_calendar_set_calls() {
        let created = answer(
            "Calendar/set",
            json!({"accountId": "a1", "newState": "s2", "created": {"c0": {"id": "K1"}}}),
        );
        let done = answer(
            "Calendar/set",
            json!({"accountId": "a1", "newState": "s2", "updated": {"K1": null},
                "destroyed": ["K1"]}),
        );
        let (mut backend, requests) = scripted(&[&created, &done, &done]);

        let id = backend
            .create_calendar(None, "Work", Some("Office"), Some("#ff0000"))
            .unwrap();
        assert_eq!(id, "K1");

        let patch = CalendarDiff {
            name: Some("Job".into()),
            description: Some(None),
            color: None,
        };
        backend.update_calendar("K1", patch).unwrap();
        backend.delete_calendar("K1").unwrap();

        let requests_text: Vec<String> = requests
            .lock()
            .unwrap()
            .iter()
            .map(|request| String::from_utf8_lossy(request).into_owned())
            .collect();
        for request in &requests_text {
            assert!(
                request.contains(
                    r#""using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:calendars"]"#
                ),
                "{request}"
            );
        }

        let sent = calls(&requests);
        assert!(sent.iter().all(|call| call[0] == "Calendar/set"));
        assert!(sent.iter().all(|call| call[1]["accountId"] == "a1"));
        assert_eq!(
            sent[0][1]["create"]["c0"],
            json!({"name": "Work", "description": "Office", "color": "#ff0000"})
        );
        assert_eq!(
            sent[1][1]["update"]["K1"],
            json!({"name": "Job", "description": null})
        );
        assert_eq!(sent[2][1]["destroy"], json!(["K1"]));
        assert_eq!(sent[2][1]["onDestroyRemoveEvents"], true);
    }

    /// A calendar the account may not create is refused before any
    /// request; one the server refuses is coded by its SetError.
    #[test]
    fn a_calendar_create_the_account_may_not_make_is_refused() {
        let (mut backend, requests) = scripted_as(&[], json!({"mayCreateCalendar": false}));
        let err = backend
            .create_calendar(None, "Work", None, None)
            .unwrap_err();
        assert_eq!(code_of(&err), Some(ErrorCode::PermissionDenied), "{err:#}");
        assert!(requests.lock().unwrap().is_empty());

        let refused = answer(
            "Calendar/set",
            json!({"accountId": "a1", "newState": "s2",
                "notCreated": {"c0": {"type": "forbidden"}}}),
        );
        let (mut backend, _) = scripted(&[&refused]);
        let err = backend
            .create_calendar(None, "Work", None, None)
            .unwrap_err();
        assert_eq!(
            crate::error::code(&err),
            Some(ErrorCode::PermissionDenied),
            "{err:#}"
        );
        assert_eq!(err.to_string(), "JMAP Calendar create rejected: forbidden");
    }
}
