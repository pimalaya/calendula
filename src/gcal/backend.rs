//! # Google Calendar backend
//!
//! The Google Calendar adapter for the shared cross-protocol client,
//! projecting [`io_gcal`]'s API v3 resources onto calendula's shared types.
//! A calendar is a calendar list entry and an item an event of it, keeping
//! the event id the API returned, verbatim.
//!
//! Google splits a calendar across two resources: the calendar carries the
//! title and the description, the per-user calendar list entry the colour,
//! so an update touches whichever of the two the patch names.
//!
//! Because the server owns the query, a [`CalendarTimeRange`] is pushed down
//! as the `timeMin` and `timeMax` parameters of `events.list` rather than
//! applied locally, and the listing walks `nextPageToken` only as far as the
//! requested window reaches.
//!
//! A series is written as its master, its EXDATEs among its recurrence
//! lines, then each override through the instance Google lists for it,
//! merged onto it and updated. What Google cannot hold (an override with a
//! RANGE, lines past what it stores) is left out with a warning naming it.
//!
//! [`CalendarTimeRange`]: crate::shared::item::CalendarTimeRange

use anyhow::{Context, Result, bail};
use ical::component::IcalComponentKind;
use io_gcal::v3::{
    client::GcalClientStd,
    rest::{
        calendar_list::{GcalCalendarListEntry, list::GcalCalendarListListParams},
        calendars::GcalCalendar,
        events::{
            GcalEvent, GcalEventStatus, GcalSendUpdates, ical::GcalEventSeries,
            import::GcalEventImportParams, insert::GcalEventInsertParams,
            instances::GcalEventInstancesParams, list::GcalEventsListParams,
            update::GcalEventUpdateParams,
        },
    },
};
use log::warn;
use pimalaya_cli::error::CodedError;
use pimalaya_config::secret::SecretResolver;

use crate::{
    config::GcalConfig,
    error::{ErrorCode, item_not_found},
    gcal::{
        client::{connect, invalid_event, item_refusal},
        render::rfc3339,
    },
    shared::{
        arg::SendUpdates,
        calendar::{Calendar, CalendarDiff},
        client::window,
        item::{CalendarItem, CalendarItemList, CalendarItemQuery},
    },
};

/// How many events to ask for per page, Google's own listing default.
const PAGE_SIZE: u32 = 250;

/// The shared-API glue over a connected Google Calendar client.
pub struct GcalBackend {
    client: GcalClientStd,
}

impl GcalBackend {
    /// Connects to the Calendar API with the configured bearer token.
    ///
    /// The token is resolved through `resolver`, so an account naming
    /// one credential command from several of its backends spawns it once.
    pub fn new(config: GcalConfig, resolver: &mut SecretResolver) -> Result<Self> {
        Ok(Self {
            client: connect(&config, resolver)?,
        })
    }

    /// Lists every calendar of the user's calendar list.
    ///
    /// Walks the pagination to the end: a calendar list is small and the
    /// unified API returns it whole.
    pub fn list_calendars(&mut self) -> Result<Vec<Calendar>> {
        let mut calendars = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let params = GcalCalendarListListParams {
                page_token: page_token.as_deref(),
                ..Default::default()
            };
            let page = self.client.calendar_list_list(&params)?.response;

            calendars.extend(page.items.into_iter().map(calendar_from));

            match page.next_page_token {
                Some(next) => page_token = Some(next),
                None => break,
            }
        }

        Ok(calendars)
    }

    /// Creates a secondary calendar, then paints it when asked for a colour.
    ///
    /// Google keeps the colour on the per-user calendar list entry, and
    /// mints the id of a calendar it creates, so the requested `id` cannot
    /// be honoured: the assigned one is returned instead.
    pub fn create_calendar(
        &mut self,
        id: Option<&str>,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<String> {
        let calendar = GcalCalendar {
            summary: Some(name.to_owned()),
            description: description.map(str::to_owned),
            ..Default::default()
        };

        let created = self.client.calendar_insert(&calendar)?.response;
        let created_id = created.id.unwrap_or_default();

        if let Some(id) = id.filter(|id| *id != created_id) {
            warn!("google assigned the calendar id `{created_id}`, not `{id}`");
        }

        if let Some(color) = color {
            self.paint(&created_id, color).with_context(|| {
                format!(
                    "Calendar `{created_id}` was created without its colour; set it with \
                     `calendar update {created_id} -C <COLOR>` rather than creating it again"
                )
            })?;
        }

        Ok(created_id)
    }

    /// Applies a partial update: title and description patch the calendar,
    /// colour patches the calendar list entry.
    ///
    /// A field a PATCH leaves out stays as it is, so a cleared description
    /// is sent empty, which Google stores as none.
    pub fn update_calendar(&mut self, id: &str, patch: CalendarDiff) -> Result<()> {
        if let Some(None) = patch.color {
            bail!("Google calendars always carry a colour; set one instead");
        }

        if patch.name.is_some() || patch.description.is_some() {
            let calendar = GcalCalendar {
                summary: patch.name,
                description: patch.description.map(Option::unwrap_or_default),
                ..Default::default()
            };

            self.client.calendar_patch(id, &calendar)?;
        }

        if let Some(Some(color)) = patch.color {
            self.paint(id, &color)?;
        }

        Ok(())
    }

    /// Deletes a secondary calendar and every event it holds.
    ///
    /// Google refuses this on a primary calendar, which surfaces as its own
    /// error.
    pub fn delete_calendar(&mut self, id: &str) -> Result<()> {
        self.client.calendar_delete(id)?;
        Ok(())
    }

    /// Lists the events of a calendar, projected onto iCalendar documents.
    ///
    /// A range narrows the query server-side, and the pagination stops as
    /// soon as the requested window is covered, counted in the resources
    /// the events fold into, so a page the caller never reaches is never
    /// fetched.
    pub fn list_items(
        &mut self,
        calendar_id: &str,
        query: CalendarItemQuery<'_>,
    ) -> Result<CalendarItemList> {
        // NOTE: Google models a VEVENT and nothing else, so any other
        // kind is answered without a round-trip rather than by filtering
        // a listing that could never hold one.
        if query
            .kind
            .is_some_and(|kind| kind != IcalComponentKind::VEvent)
        {
            return Ok(CalendarItemList::default());
        }

        let range = query.range;
        let time_min = range.and_then(|range| range.start.as_deref()).map(rfc3339);
        let time_max = range.and_then(|range| range.end.as_deref()).map(rfc3339);
        let wanted = query.take.map(|take| query.skip.saturating_add(take));

        let mut events = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let params = GcalEventsListParams {
                max_results: Some(PAGE_SIZE),
                page_token: page_token.as_deref(),
                time_min: time_min.as_deref(),
                time_max: time_max.as_deref(),
                ..Default::default()
            };
            let current = self.client.events_list(calendar_id, &params)?.response;

            events.extend(current.items);

            if wanted.is_some_and(|wanted| item_count(&events) >= wanted) {
                break;
            }

            match current.next_page_token {
                Some(next) => page_token = Some(next),
                None => break,
            }
        }

        let items = resources(calendar_id, &events);
        Ok(CalendarItemList {
            items: window(items, query.skip, query.take),
            errors: Vec::new(),
        })
    }

    /// Reads one event, with the exceptions of a series folded into it.
    pub fn get_item(&mut self, calendar_id: &str, item_id: &str) -> Result<CalendarItem> {
        let event = self
            .event(calendar_id, item_id)
            .map_err(|err| crate::error::read_failed(err, item_id, calendar_id))?;

        let exceptions = self.exceptions(calendar_id, &event)?;

        Ok(item_from(
            calendar_id,
            &event,
            &exceptions.iter().collect::<Vec<_>>(),
        ))
    }

    /// The event `item_id` of `calendar_id`, `item-not-found` when Google
    /// holds none or a cancelled one: a deleted event stays as a cancelled
    /// stub its reads return and its listings leave out.
    fn event(&mut self, calendar_id: &str, item_id: &str) -> Result<GcalEvent> {
        let event = self
            .client
            .event_get(calendar_id, item_id, None, None)
            .map_err(item_refusal)?
            .response;

        if event.status == Some(GcalEventStatus::Cancelled) {
            return Err(item_not_found(format!(
                "Google holds event `{item_id}` as cancelled, a deleted event"
            )));
        }

        Ok(event)
    }

    /// The exceptions of a series: the events Google returns beside a
    /// recurring event to say one of its instances was modified.
    ///
    /// One listing filtered on the series' iCalUID returns master and
    /// exceptions, since the API offers no "children of this event" query
    /// and expanding the instances would return the occurrences the RRULE
    /// generates rather than the modifications alone.
    fn exceptions(&mut self, calendar_id: &str, event: &GcalEvent) -> Result<Vec<GcalEvent>> {
        if event.recurrence.is_empty() {
            return Ok(Vec::new());
        }

        let Some(uid) = event.ical_uid.as_deref() else {
            return Ok(Vec::new());
        };

        let mut exceptions = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let params = GcalEventsListParams {
                max_results: Some(PAGE_SIZE),
                page_token: page_token.as_deref(),
                ical_uid: Some(uid),
                ..Default::default()
            };
            let current = self.client.events_list(calendar_id, &params)?.response;

            exceptions.extend(
                current
                    .items
                    .into_iter()
                    .filter(|listed| listed.recurring_event_id.as_deref() == event.id.as_deref()),
            );

            match current.next_page_token {
                Some(next) => page_token = Some(next),
                None => break,
            }
        }

        Ok(exceptions)
    }

    /// Creates an event from an iCalendar document.
    ///
    /// An event the account organizes and invites people to is inserted
    /// with `sendUpdates=all`, its UID sent as the `iCalUID` so it survives,
    /// for Google to mail the attendees. Any other document carrying a UID
    /// is imported, so the UID survives as the event's `iCalUID` and the
    /// resource keeps the identity it already had; one carrying none is
    /// inserted, and Google mints both ids itself.
    ///
    /// A `send_updates` given replaces the `all` of an inserted event. An
    /// import mails nobody and takes no `sendUpdates`, so asking it to
    /// mail the attendees is refused, naming the organizer.
    ///
    /// A UID the calendar already holds is refused `item-duplicate`
    /// before anything is written: an import matches on it and would
    /// overwrite that event.
    ///
    /// A series' overrides follow the master, written as its write mails;
    /// one that fails deletes the series again, so a failed create leaves
    /// nothing half written.
    pub fn create_item(
        &mut self,
        calendar_id: &str,
        contents: Vec<u8>,
        send_updates: Option<SendUpdates>,
    ) -> Result<String> {
        let series = project(&contents)?;
        let event = &series.master;
        let path = create_path(event, calendar_id, send_updates)?;

        if let Some(uid) = event.ical_uid.as_deref().filter(|uid| !uid.is_empty())
            && let Some(id) = self.holding(calendar_id, uid)?
        {
            let message = format!(
                "Calendar `{calendar_id}` already holds `{uid}` as event `{id}`; update that \
                 event instead"
            );
            return Err(CodedError::new(ErrorCode::ItemDuplicate, message).into());
        }

        // NOTE: an import mails nobody, so neither do its overrides.
        let (created, mails) = match path {
            CreatePath::Insert(send_updates) => {
                let params = GcalEventInsertParams {
                    send_updates,
                    ..Default::default()
                };
                let created = self
                    .client
                    .event_insert(calendar_id, event, &params)
                    .map_err(|err| invalid_event(err).unwrap_or_else(Into::into))?;
                (created, send_updates)
            }
            CreatePath::Import => {
                let params = GcalEventImportParams::default();
                let created = self
                    .client
                    .event_import(calendar_id, event, &params)
                    .map_err(|err| invalid_event(err).unwrap_or_else(Into::into))?;
                (created, Some(GcalSendUpdates::None))
            }
        };

        let id = created.response.id.unwrap_or_default();

        if let Err(err) = self.change_instances(calendar_id, &id, &series.instances, mails) {
            if let Err(undo) = self.client.event_delete(calendar_id, &id, mails, None) {
                warn!("cannot delete the series `{id}` again: {undo}");
            }
            return Err(err.context("Cannot create the occurrences of the series"));
        }

        Ok(id)
    }

    /// The id of the event of `calendar_id` carrying `uid`, the series
    /// master when it holds one, `None` when it holds none.
    ///
    /// A listing filtered on the `iCalUID` leaves the deleted events out,
    /// and may answer an empty page before the one holding a match.
    fn holding(&mut self, calendar_id: &str, uid: &str) -> Result<Option<String>> {
        let mut page_token: Option<String> = None;

        loop {
            let params = GcalEventsListParams {
                max_results: Some(PAGE_SIZE),
                page_token: page_token.as_deref(),
                ical_uid: Some(uid),
                ..Default::default()
            };
            let current = self.client.events_list(calendar_id, &params)?.response;

            let master = current
                .items
                .iter()
                .find(|event| event.recurring_event_id.is_none())
                .or(current.items.first());

            if let Some(event) = master {
                return Ok(Some(event.id.clone().unwrap_or_default()));
            }

            match current.next_page_token {
                Some(next) => page_token = Some(next),
                None => return Ok(None),
            }
        }
    }

    /// Replaces an event from an iCalendar document, optionally gated on
    /// an `if_match` etag.
    ///
    /// The current server event is read first and serves as the base the
    /// projection merges onto, so the fields no iCalendar property models
    /// survive a write that would otherwise clear them.
    pub fn update_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
        send_updates: Option<SendUpdates>,
    ) -> Result<()> {
        let series = project(&contents)?;
        let current = self.event(calendar_id, item_id)?;

        let event = series.master.merge(&current);
        let send_updates = send_updates.map(gcal_send_updates);
        let params = GcalEventUpdateParams {
            send_updates,
            ..Default::default()
        };

        self.client
            .event_update(calendar_id, item_id, &event, &params, if_match)
            .map_err(|err| invalid_event(err).unwrap_or_else(item_refusal))?;

        self.change_instances(calendar_id, item_id, &series.instances, send_updates)
            .context("The series is updated, but not all of its occurrences")
    }

    /// Writes each override of `overrides` onto the instance of the series
    /// `master_id` Google lists for it, merged onto it and gated on its
    /// etag, mailing whom `send_updates` names.
    ///
    /// An override no listing holds is cancelled already when it says so,
    /// and names an occurrence the series does not have otherwise.
    fn change_instances(
        &mut self,
        calendar_id: &str,
        master_id: &str,
        overrides: &[GcalEvent],
        send_updates: Option<GcalSendUpdates>,
    ) -> Result<()> {
        let params = GcalEventUpdateParams {
            send_updates,
            ..Default::default()
        };

        for instance in overrides {
            let original = instance
                .original_start_time
                .as_ref()
                .and_then(|start| start.date_time.as_deref().or(start.date.as_deref()))
                .unwrap_or_default();
            let Some(window) = instance.instance_window() else {
                bail!("An override of series `{master_id}` names no readable occurrence");
            };

            let Some(listed) = self.instance(calendar_id, master_id, &window, instance)? else {
                if instance.status == Some(GcalEventStatus::Cancelled) {
                    continue;
                }
                bail!("Series `{master_id}` has no occurrence starting at {original}");
            };

            let id = listed.id.clone().unwrap_or_default();
            let merged = instance.clone().merge(&listed);
            self.client
                .event_update(calendar_id, &id, &merged, &params, listed.etag.as_deref())
                .map_err(|err| invalid_event(err).unwrap_or_else(item_refusal))?;
        }

        Ok(())
    }

    /// The instance of the series `master_id` that `instance` replaces,
    /// listed within `window`.
    fn instance(
        &mut self,
        calendar_id: &str,
        master_id: &str,
        window: &(String, String),
        instance: &GcalEvent,
    ) -> Result<Option<GcalEvent>> {
        let mut page_token: Option<String> = None;

        loop {
            let params = GcalEventInstancesParams {
                time_min: Some(&window.0),
                time_max: Some(&window.1),
                page_token: page_token.as_deref(),
                ..Default::default()
            };
            let page = self
                .client
                .event_instances(calendar_id, master_id, &params)
                .map_err(item_refusal)?
                .response;

            let listed = page
                .items
                .into_iter()
                .find(|listed| instance.replaces(listed));
            if listed.is_some() {
                return Ok(listed);
            }

            match page.next_page_token {
                Some(next) => page_token = Some(next),
                None => return Ok(None),
            }
        }
    }

    /// Deletes an event, optionally gated on an `if_match` etag, mailing
    /// whom `send_updates` names, Google deciding when it names nobody.
    pub fn delete_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        if_match: Option<&str>,
        send_updates: Option<SendUpdates>,
    ) -> Result<()> {
        let send_updates = send_updates.map(gcal_send_updates);
        self.client
            .event_delete(calendar_id, item_id, send_updates, if_match)
            .map_err(item_refusal)?;
        Ok(())
    }

    /// Sets the background colour of a calendar through its calendar list
    /// entry, the only resource carrying one.
    fn paint(&mut self, id: &str, color: &str) -> Result<()> {
        let entry = GcalCalendarListEntry {
            background_color: Some(color.to_owned()),
            ..Default::default()
        };

        self.client
            .calendar_list_entry_patch(id, &entry, Some(true))?;

        Ok(())
    }
}

/// Projects an iCalendar document onto the series a write sends: its
/// master and its overrides.
///
/// What Google cannot hold (lines past what it stores beside an event, an
/// override with a RANGE) is left out with a warning naming it, the write
/// going on.
fn project(contents: &[u8]) -> Result<GcalEventSeries> {
    let series = GcalEventSeries::from_ical(contents)?;

    for unit in &series.left_out {
        warn!(
            "writing the event without some of its lines, which Google cannot hold: {}",
            unit.join(" ")
        );
    }

    Ok(series)
}

/// How a create reaches Google.
#[derive(Debug, Eq, PartialEq)]
enum CreatePath {
    /// `events.insert`, with this `sendUpdates`.
    Insert(Option<GcalSendUpdates>),
    /// `events.import`, which mails nobody.
    Import,
}

/// How `event` is created in `calendar_id`, `send_updates` being what
/// the command was given.
///
/// An event the account invites people to is inserted mailing them, or
/// mailing whom `send_updates` names; one carrying a UID otherwise is
/// imported, and one carrying none is inserted. An import mails nobody,
/// so `all` or `external-only` asked of one with attendees is refused,
/// naming its organizer, rather than ignored.
fn create_path(
    event: &GcalEvent,
    calendar_id: &str,
    send_updates: Option<SendUpdates>,
) -> Result<CreatePath> {
    let has_uid = event.ical_uid.as_deref().is_some_and(|uid| !uid.is_empty());

    if invites(event, calendar_id) {
        let send_updates = send_updates.map_or(GcalSendUpdates::All, gcal_send_updates);
        return Ok(CreatePath::Insert(Some(send_updates)));
    }

    if !has_uid {
        return Ok(CreatePath::Insert(send_updates.map(gcal_send_updates)));
    }

    let mails = matches!(
        send_updates,
        Some(SendUpdates::All | SendUpdates::ExternalOnly)
    );

    if mails && !event.attendees.is_empty() {
        let organizer = event
            .organizer
            .as_ref()
            .and_then(|organizer| organizer.email.as_deref())
            .unwrap_or_default();

        bail!(
            "Cannot mail the attendees of an event organized by {organizer}: Google imports it \
             and mails nobody; give --send-updates none or leave it out"
        );
    }

    Ok(CreatePath::Import)
}

/// The io-gcal spelling of whom a write mails.
fn gcal_send_updates(send_updates: SendUpdates) -> GcalSendUpdates {
    match send_updates {
        SendUpdates::All => GcalSendUpdates::All,
        SendUpdates::ExternalOnly => GcalSendUpdates::ExternalOnly,
        SendUpdates::None => GcalSendUpdates::None,
    }
}

/// Whether creating `event` in `calendar_id` invites its attendees: it
/// names at least one, and either no organizer, which Google then makes
/// the account, or the calendar itself, the address of a primary calendar.
///
/// An event organized by anyone else is a copy of their invitation, or a
/// migration, and inserting it would make the account mail its guests on
/// the organizer's behalf.
fn invites(event: &GcalEvent, calendar_id: &str) -> bool {
    if event.attendees.is_empty() {
        return false;
    }

    match &event.organizer {
        None => true,
        Some(organizer) => organizer
            .email
            .as_deref()
            .is_some_and(|email| email.eq_ignore_ascii_case(calendar_id)),
    }
}

/// Projects a calendar list entry onto the shared [`Calendar`], preferring
/// the title this user gave the calendar over its own.
///
/// The user's primary calendar is the default one, and an empty
/// description, what a clear stores, is none.
fn calendar_from(entry: GcalCalendarListEntry) -> Calendar {
    let id = entry.id.unwrap_or_default();

    Calendar {
        name: entry
            .summary_override
            .or(entry.summary)
            .unwrap_or_else(|| id.clone()),
        id,
        description: entry.description.filter(|text| !text.is_empty()),
        color: entry.background_color,
        default: entry.primary.unwrap_or_default(),
    }
}

/// Folds a batch of Google events into calendar object resources.
///
/// An exception is not an item of its own: RFC 4791 4.1 keeps components
/// sharing a UID in one resource. One whose master this batch did not
/// return still files alone, since dropping it would lose a listed item.
fn resources(calendar_id: &str, events: &[GcalEvent]) -> Vec<CalendarItem> {
    let masters = masters(events);

    events
        .iter()
        .filter(|event| !folds(event, &masters))
        .map(|event| {
            let exceptions: Vec<&GcalEvent> = events
                .iter()
                .filter(|listed| {
                    event.id.is_some()
                        && listed.recurring_event_id.as_deref() == event.id.as_deref()
                })
                .collect();

            item_from(calendar_id, event, &exceptions)
        })
        .collect()
}

/// How many resources a batch of Google events folds into, counted
/// without projecting them as [`resources`] does.
fn item_count(events: &[GcalEvent]) -> usize {
    let masters = masters(events);
    let items = events.iter().filter(|event| !folds(event, &masters));
    items.count()
}

/// The ids of the series masters a batch of Google events holds.
fn masters(events: &[GcalEvent]) -> Vec<&str> {
    events
        .iter()
        .filter(|event| event.recurring_event_id.is_none())
        .filter_map(|event| event.id.as_deref())
        .collect()
}

/// Whether `event` is an exception of one of `masters`, folded into that
/// series' resource rather than filed alone.
fn folds(event: &GcalEvent, masters: &[&str]) -> bool {
    event
        .recurring_event_id
        .as_deref()
        .is_some_and(|master| masters.contains(&master))
}

/// Projects an event and the exceptions of its series onto the shared
/// [`CalendarItem`], its contents synthesized by the projection.
fn item_from(calendar_id: &str, event: &GcalEvent, exceptions: &[&GcalEvent]) -> CalendarItem {
    CalendarItem {
        id: event.id.clone().unwrap_or_default(),
        calendar_id: calendar_id.to_owned(),
        etag: event.etag.clone(),
        contents: event.to_ical_series(exceptions).into_bytes(),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        io::{Read, Result as IoResult, Write},
    };

    use super::*;
    use io_gcal::v3::rest::events::{GcalEventAttendee, GcalEventDateTime, GcalEventPerson};

    use crate::shared::event::Event;

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

    /// A backend whose server answers 200 with each of `bodies` in turn.
    fn backend(bodies: &[&str]) -> GcalBackend {
        let response = |body: &&str| {
            let len = body.len();
            format!("HTTP/1.1 200 OK\r\nContent-Length: {len}\r\n\r\n{body}").into_bytes()
        };
        let stream = Scripted {
            responses: bodies.iter().map(response).collect(),
            written: Vec::new(),
        };

        GcalBackend {
            client: GcalClientStd::new(stream, "made-up"),
        }
    }

    /// A backend whose server answers each `(status, body)` in turn.
    fn answering(answers: &[(u16, &str)]) -> GcalBackend {
        let response = |(status, body): &(u16, &str)| {
            let len = body.len();
            format!("HTTP/1.1 {status} Answer\r\nContent-Length: {len}\r\n\r\n{body}").into_bytes()
        };
        let stream = Scripted {
            responses: answers.iter().map(response).collect(),
            written: Vec::new(),
        };

        GcalBackend {
            client: GcalClientStd::new(stream, "made-up"),
        }
    }

    /// A Calendar error envelope of `status`, naming `reason`.
    fn refusal(status: u16, reason: &str, message: &str) -> String {
        format!(
            r#"{{"error":{{"code":{status},"message":"{message}","errors":[{{"reason":"{reason}","message":"{message}"}}]}}}}"#
        )
    }

    /// What `backend` wrote to its scripted server.
    fn written(backend: &mut GcalBackend) -> String {
        let stream = backend.client.stream.as_any_mut();
        let stream = stream.downcast_mut::<Scripted>().unwrap();
        String::from_utf8_lossy(&stream.written).into_owned()
    }

    /// CAL-11: a refusal knowable before any request sends none.
    #[test]
    fn a_cleared_colour_is_refused_before_the_rename() {
        let mut backend = backend(&[]);
        let patch = CalendarDiff {
            name: Some("New".into()),
            color: Some(None),
            ..Default::default()
        };

        let err = backend.update_calendar("cal", patch).unwrap_err();

        assert!(err.to_string().contains("colour"), "{err:#}");
        assert_eq!(written(&mut backend), "");
    }

    /// CAL-11: a calendar created then left unpainted is named, so a
    /// retry updates it rather than creating a second one.
    #[test]
    fn a_failed_paint_names_the_calendar_created() {
        let mut backend = answering(&[
            (
                200,
                r#"{"id":"new@group.calendar.google.com","summary":"N"}"#,
            ),
            (400, &refusal(400, "invalid", "Invalid colour")),
        ]);

        let err = backend
            .create_calendar(None, "N", None, Some("nope"))
            .unwrap_err();

        assert!(
            err.to_string()
                .contains("Calendar `new@group.calendar.google.com` was created"),
            "{err:#}"
        );
    }

    #[test]
    fn an_id_holding_a_hash_or_an_at_names_its_own_resource() {
        let calendar_id = "en.usa#holiday@group.v.calendar.google.com";
        let mut backend = backend(&[r#"{"id":"2026#1@x","etag":"\"1\""}"#, ""]);

        let item = backend.get_item(calendar_id, "2026#1@x").unwrap();
        assert_eq!(item.id, "2026#1@x");
        assert_eq!(item.calendar_id, calendar_id);

        backend
            .delete_item(calendar_id, "2026#1@x", None, None)
            .unwrap();

        let path = "/calendar/v3/calendars/en.usa%23holiday@group.v.calendar.google.com/events/\
                    2026%231@x HTTP/1.1\r\n";
        let requests = written(&mut backend);
        assert!(requests.contains(&format!("GET {path}")), "{requests}");
        assert!(requests.contains(&format!("DELETE {path}")), "{requests}");
    }

    #[test]
    fn a_stash_over_what_google_stores_is_written_without_the_lines_left_out() {
        let lines: Vec<String> = (0..40)
            .map(|index| format!("X-LINE-{index}:{}\r\n", "a".repeat(1000)))
            .collect();
        let calendar = format!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\nBEGIN:VEVENT\r\n\
             UID:1@example.org\r\nDTSTAMP:20260101T000000Z\r\nDTSTART:20260814T090000Z\r\n\
             DTEND:20260814T100000Z\r\n{}END:VEVENT\r\nEND:VCALENDAR\r\n",
            lines.concat(),
        );

        let mut backend = backend(&[r#"{"items":[]}"#, r#"{"id":"minted"}"#]);
        let id = backend.create_item("primary", calendar.into_bytes(), None);
        assert_eq!(id.unwrap(), "minted");

        let requests = written(&mut backend);
        assert!(requests.contains("X-LINE-0:"), "{requests}");
        assert!(!requests.contains("X-LINE-39:"), "{requests}");
    }

    /// An event carrying a UID the calendar holds is refused before
    /// anything is written: `events.import` would overwrite it.
    #[test]
    fn a_create_naming_a_held_uid_is_refused_and_imports_nothing() {
        use crate::error::code;

        let calendar = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\nBEGIN:VEVENT\r\n\
                        UID:e1@example.org\r\nDTSTAMP:20260101T000000Z\r\n\
                        DTSTART:20260814T090000Z\r\nDTEND:20260814T100000Z\r\n\
                        SUMMARY:Another\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let held = r#"{"items":[{"id":"held1","iCalUID":"e1@example.org"}]}"#;

        let mut backend = backend(&[r#"{"items":[],"nextPageToken":"2"}"#, held]);
        let err = backend
            .create_item("primary", calendar.as_bytes().to_vec(), None)
            .unwrap_err();
        assert_eq!(code(&err), Some(ErrorCode::ItemDuplicate), "{err:#}");
        assert!(err.to_string().contains("as event `held1`"), "{err}");

        let requests = written(&mut backend);
        assert_eq!(requests.matches("GET ").count(), 2, "{requests}");
        assert!(requests.contains("iCalUID=e1%40example.org"), "{requests}");
        assert!(!requests.contains("POST "), "{requests}");
    }

    /// An event write answered 400 refuses the event sent, every parameter
    /// being calendula's: `item-invalid`, on an import, an insert and an
    /// update alike; a calendar the write names that Google does not hold
    /// keeps no code, and a listing answered 400 neither.
    #[test]
    fn an_event_write_answered_400_is_invalid() {
        use crate::error::code;

        let event = |uid: &str| {
            format!(
                "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\nBEGIN:VEVENT\r\n\
                 {uid}DTSTAMP:20260101T000000Z\r\nDTSTART:20260814T090000Z\r\n\
                 DTEND:20260814T080000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
            )
            .into_bytes()
        };
        let refused = refusal(400, "timeRangeEmpty", "The specified time range is empty.");
        let missing = refusal(404, "notFound", "Not Found");
        let empty = r#"{"items":[]}"#;
        let current = r#"{"id":"e1","etag":"\"1\"","iCalUID":"e1@example.org"}"#;

        let err = answering(&[(200, empty), (400, &refused)])
            .create_item("primary", event("UID:e1@example.org\r\n"), None)
            .unwrap_err();
        assert_eq!(code(&err), Some(ErrorCode::ItemInvalid), "{err:#}");
        assert!(err.to_string().contains("time range is empty"), "{err}");

        let err = answering(&[(400, &refused)])
            .create_item("primary", event(""), None)
            .unwrap_err();
        assert_eq!(code(&err), Some(ErrorCode::ItemInvalid), "{err:#}");

        let err = answering(&[(200, current), (400, &refused)])
            .update_item("primary", "e1", event("UID:e1@example.org\r\n"), None, None)
            .unwrap_err();
        assert_eq!(code(&err), Some(ErrorCode::ItemInvalid), "{err:#}");

        let err = answering(&[(404, &missing)])
            .create_item("nope", event(""), None)
            .unwrap_err();
        assert_eq!(code(&err), None, "{err:#}");

        let err = answering(&[(400, &refused)]).list_calendars().unwrap_err();
        assert_eq!(code(&err), None, "{err:#}");
    }

    #[test]
    fn an_exception_files_with_its_master_rather_than_beside_it() {
        let event = |id: &str, master: Option<&str>| GcalEvent {
            id: Some(id.to_owned()),
            ical_uid: Some(format!("{id}@example.org")),
            recurring_event_id: master.map(str::to_owned),
            ..Default::default()
        };

        let events = vec![
            event("standup", None),
            event("standup_20260811T090000Z", Some("standup")),
            event("lunch", None),
            // NOTE: its master fell outside the window this page asked for,
            // so it stands on its own rather than being dropped.
            event("orphan_20260811T090000Z", Some("elsewhere")),
        ];

        let items = resources("personal", &events);
        let ids: Vec<&str> = items.iter().map(|item| item.id.as_str()).collect();

        assert_eq!(ids, ["standup", "lunch", "orphan_20260811T090000Z"]);

        let series = String::from_utf8(items[0].contents.clone()).unwrap();
        assert_eq!(series.matches("BEGIN:VEVENT").count(), 2);
    }

    #[test]
    fn the_synthesized_document_feeds_the_shared_event_projection() {
        let at = |stamp: &str| GcalEventDateTime {
            date_time: Some(stamp.to_owned()),
            ..Default::default()
        };
        let event = GcalEvent {
            id: Some(String::from("event-1")),
            summary: Some(String::from("Stand-up")),
            start: Some(at("2026-08-14T09:00:00Z")),
            end: Some(at("2026-08-14T10:00:00Z")),
            ..Default::default()
        };

        let events = Event::project(&item_from("primary", &event, &[]));

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].summary.as_deref(), Some("Stand-up"));
        assert_eq!(events[0].start.as_deref(), Some("2026-08-14T09:00:00Z"));
        assert_eq!(events[0].end.as_deref(), Some("2026-08-14T10:00:00Z"));
    }

    /// An exception whose master the batch holds folds into it, one whose
    /// master it lacks counts alone.
    #[test]
    fn exceptions_count_with_their_master() {
        let event = |id: &str, master: Option<&str>| GcalEvent {
            id: Some(id.to_owned()),
            recurring_event_id: master.map(str::to_owned),
            ..Default::default()
        };
        let events = [
            event("a", None),
            event("a_1", Some("a")),
            event("a_2", Some("a")),
            event("b_1", Some("b")),
        ];

        assert_eq!(item_count(&events), 2);
        assert_eq!(resources("cal", &events).len(), 2);
    }

    /// An event organized by `organizer`, inviting `attendees`, carrying
    /// a UID.
    fn invitation(organizer: Option<&str>, attendees: &[&str]) -> GcalEvent {
        GcalEvent {
            ical_uid: Some(String::from("1@example.org")),
            organizer: organizer.map(|email| GcalEventPerson {
                email: Some(email.to_owned()),
                ..Default::default()
            }),
            attendees: attendees
                .iter()
                .map(|email| GcalEventAttendee {
                    email: Some((*email).to_owned()),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn send_updates_replaces_the_all_of_an_invitation() {
        let event = invitation(None, &["ann@example.org"]);
        let path = |send_updates| create_path(&event, "me@example.org", send_updates).unwrap();

        assert_eq!(path(None), CreatePath::Insert(Some(GcalSendUpdates::All)));
        assert_eq!(
            path(Some(SendUpdates::All)),
            CreatePath::Insert(Some(GcalSendUpdates::All))
        );
        assert_eq!(
            path(Some(SendUpdates::ExternalOnly)),
            CreatePath::Insert(Some(GcalSendUpdates::ExternalOnly))
        );
        assert_eq!(
            path(Some(SendUpdates::None)),
            CreatePath::Insert(Some(GcalSendUpdates::None))
        );
    }

    #[test]
    fn an_imported_invitation_mails_nobody_and_refuses_to_mail() {
        let event = invitation(Some("bob@example.org"), &["ann@example.org"]);
        let path = |send_updates| create_path(&event, "me@example.org", send_updates);

        assert_eq!(path(None).unwrap(), CreatePath::Import);
        assert_eq!(path(Some(SendUpdates::None)).unwrap(), CreatePath::Import);

        for send_updates in [SendUpdates::All, SendUpdates::ExternalOnly] {
            let err = path(Some(send_updates)).unwrap_err().to_string();
            assert!(err.contains("organized by bob@example.org"), "{err}");
        }
    }

    #[test]
    fn an_event_inviting_nobody_takes_any_send_updates() {
        let event = invitation(Some("bob@example.org"), &[]);

        for send_updates in [None, Some(SendUpdates::All), Some(SendUpdates::None)] {
            let path = create_path(&event, "me@example.org", send_updates).unwrap();
            assert_eq!(path, CreatePath::Import);
        }

        // NOTE: without a UID the event is inserted, and the choice
        // travels as given, or not at all.
        let bare = GcalEvent::default();
        assert_eq!(
            create_path(&bare, "me@example.org", None).unwrap(),
            CreatePath::Insert(None)
        );
        assert_eq!(
            create_path(&bare, "me@example.org", Some(SendUpdates::None)).unwrap(),
            CreatePath::Insert(Some(GcalSendUpdates::None))
        );
    }

    #[test]
    fn only_an_event_the_account_organizes_invites_its_attendees() {
        let event = |organizer: Option<&str>, attendees: &[&str]| GcalEvent {
            organizer: organizer.map(|email| GcalEventPerson {
                email: Some(email.to_owned()),
                ..Default::default()
            }),
            attendees: attendees
                .iter()
                .map(|email| GcalEventAttendee {
                    email: Some((*email).to_owned()),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        let calendar = "me@example.org";

        assert!(invites(&event(None, &["ann@example.org"]), calendar));
        assert!(invites(
            &event(Some("me@example.org"), &["ann@example.org"]),
            calendar
        ));
        assert!(invites(
            &event(Some("Me@Example.org"), &["ann@example.org"]),
            calendar
        ));

        assert!(!invites(&event(None, &[]), calendar));
        assert!(!invites(&event(Some("me@example.org"), &[]), calendar));
        assert!(!invites(
            &event(Some("bob@example.org"), &["ann@example.org"]),
            calendar
        ));
    }

    #[test]
    fn a_refusal_carries_its_code() {
        use crate::error::{ErrorCode, code};

        let cases = [
            (
                403,
                "insufficientPermissions",
                "Insufficient Permission",
                Some(ErrorCode::PermissionDenied),
            ),
            (
                403,
                "rateLimitExceeded",
                "Rate Limit Exceeded",
                Some(ErrorCode::RateLimited),
            ),
            (
                403,
                "quotaExceeded",
                "Calendar usage limits exceeded.",
                Some(ErrorCode::RateLimited),
            ),
            (
                403,
                "dailyLimitExceeded",
                "Daily Limit Exceeded",
                Some(ErrorCode::RateLimited),
            ),
            (
                403,
                "dailyLimitExceededUnreg",
                "Daily Limit for Unauthenticated Use Exceeded.",
                Some(ErrorCode::PermissionDenied),
            ),
            (
                429,
                "rateLimitExceeded",
                "Too Many Requests",
                Some(ErrorCode::RateLimited),
            ),
            (
                412,
                "conditionNotMet",
                "Precondition Failed",
                Some(ErrorCode::PreconditionFailed),
            ),
            (500, "backendError", "Backend Error", None),
        ];

        for (status, reason, message, expected) in cases {
            let body = refusal(status, reason, message);
            let err = answering(&[(status, &body)]).list_calendars().unwrap_err();
            assert_eq!(code(&err), expected, "{status} {reason}: {err:#}");
        }
    }

    #[test]
    fn a_stale_if_match_and_a_missing_event_carry_their_code() {
        use crate::error::{ErrorCode, code};

        let body = refusal(412, "conditionNotMet", "Precondition Failed");
        let err = answering(&[(412, &body)])
            .delete_item("primary", "e1", Some("\"1\""), None)
            .unwrap_err();
        assert_eq!(code(&err), Some(ErrorCode::PreconditionFailed), "{err:#}");

        for status in [404, 410] {
            let body = refusal(status, "notFound", "Not Found");
            let err = answering(&[(status, &body)])
                .get_item("primary", "nope")
                .unwrap_err();
            assert_eq!(code(&err), Some(ErrorCode::ItemNotFound), "{err:#}");
        }

        let body = refusal(404, "notFound", "Not Found");
        let err = answering(&[(404, &body)]).list_calendars().unwrap_err();
        assert_eq!(code(&err), None, "a calendar request: {err:#}");
    }

    /// A deleted event, which Google keeps as a cancelled stub its read
    /// returns, is not found, as on every other backend.
    #[test]
    fn a_cancelled_event_is_not_found() {
        use crate::error::code;

        let stub = r#"{"id":"e1","status":"cancelled","iCalUID":"e1@example.org"}"#;
        let err = backend(&[stub]).get_item("primary", "e1").unwrap_err();
        assert_eq!(code(&err), Some(ErrorCode::ItemNotFound), "{err:#}");
        assert!(!format!("{err:#}").contains("Read item"), "{err:#}");
    }

    /// A cleared description is sent empty, a field the PATCH leaves out
    /// staying as it was, and an empty one reads as none.
    #[test]
    fn a_cleared_description_is_sent_empty() {
        let mut backend = backend(&[r#"{"id":"cal"}"#]);
        let patch = CalendarDiff {
            description: Some(None),
            ..Default::default()
        };
        backend.update_calendar("cal", patch).unwrap();

        let requests = written(&mut backend);
        assert!(requests.starts_with("PATCH "), "{requests}");
        assert!(requests.contains(r#""description":"""#), "{requests}");

        let entry = GcalCalendarListEntry {
            id: Some(String::from("cal")),
            description: Some(String::new()),
            ..Default::default()
        };
        assert_eq!(calendar_from(entry).description, None);
    }

    /// A weekly series cancelling its second occurrence (an EXDATE) and
    /// moving its third (an override).
    const SERIES: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\n\
        BEGIN:VEVENT\r\nUID:s@example.org\r\nDTSTAMP:20261101T000000Z\r\n\
        DTSTART:20261102T090000Z\r\nDTEND:20261102T093000Z\r\n\
        RRULE:FREQ=WEEKLY;COUNT=4\r\nEXDATE:20261109T090000Z\r\nSUMMARY:Standup\r\n\
        END:VEVENT\r\nBEGIN:VEVENT\r\nUID:s@example.org\r\nDTSTAMP:20261101T000000Z\r\n\
        RECURRENCE-ID:20261116T090000Z\r\nDTSTART:20261116T100000Z\r\n\
        DTEND:20261116T103000Z\r\nSUMMARY:Standup moved\r\nEND:VEVENT\r\n\
        END:VCALENDAR\r\n";

    /// A series is imported with its EXDATE, then its override written onto
    /// the instance Google lists for it, gated on that instance's etag,
    /// where the master alone was written.
    #[test]
    fn a_series_creates_its_exceptions() {
        let instances = r#"{"items":[{"id":"M_20261116T090000Z","etag":"\"e3\"",
            "recurringEventId":"M","originalStartTime":{"dateTime":"2026-11-16T09:00:00Z"},
            "start":{"dateTime":"2026-11-16T09:00:00Z"},"end":{"dateTime":"2026-11-16T09:30:00Z"}}]}"#;
        let mut backend = backend(&[
            r#"{"items":[]}"#,
            r#"{"id":"M"}"#,
            instances,
            r#"{"id":"M_20261116T090000Z"}"#,
        ]);

        let id = backend
            .create_item("primary", SERIES.as_bytes().to_vec(), None)
            .unwrap();
        assert_eq!(id, "M");

        let sent = written(&mut backend);
        let import = sent
            .find("POST /calendar/v3/calendars/primary/events/import")
            .unwrap();
        let listed = sent
            .find("GET /calendar/v3/calendars/primary/events/M/instances")
            .unwrap();
        let updated = sent
            .find("PUT /calendar/v3/calendars/primary/events/M_20261116T090000Z")
            .unwrap();
        assert!(import < listed && listed < updated, "{sent}");
        assert!(sent.contains("EXDATE"), "{sent}");
        assert!(sent.contains("Standup moved"), "{sent}");
        assert!(sent[updated..].contains("If-Match: \"e3\""), "{sent}");
    }

    /// A weekly series at 09:00 in Paris read from a calendar in UTC keeps
    /// 09:00 across the daylight-saving change, as Google expands it, where
    /// the projection wrote its first instant in UTC and the occurrences
    /// after the change fell at 10:00 (io-gcal 0.5.1).
    #[test]
    fn a_zoned_series_keeps_its_wall_time_across_dst() {
        let event = r#"{"id":"w","status":"confirmed","iCalUID":"w@example.org",
            "start":{"dateTime":"2027-03-15T08:00:00Z","timeZone":"Europe/Paris"},
            "end":{"dateTime":"2027-03-15T08:30:00Z","timeZone":"Europe/Paris"},
            "recurrence":["RRULE:FREQ=WEEKLY;COUNT=3"]}"#;
        let mut backend = backend(&[event, r#"{"items":[]}"#]);

        let item = backend.get_item("primary", "w").unwrap();
        let contents = String::from_utf8_lossy(&item.contents).into_owned();
        assert!(
            contents.contains("DTSTART;TZID=Europe/Paris:20270315T090000"),
            "{contents}"
        );

        let starts: Vec<String> = crate::shared::event::expand::occurrences(&item, None)
            .unwrap()
            .into_iter()
            .filter_map(|event| event.start)
            .collect();
        assert_eq!(
            starts,
            [
                "2027-03-15T09:00:00+01:00",
                "2027-03-22T09:00:00+01:00",
                "2027-03-29T09:00:00+02:00",
            ]
        );
    }

    /// The user's primary calendar is the default one, as `gcal
    /// calendars` marks it.
    #[test]
    fn the_primary_calendar_is_the_default() {
        let list = r#"{"items":[{"id":"me@example.org","summary":"Me","primary":true},
                                {"id":"other","summary":"Other"}]}"#;
        let calendars = backend(&[list]).list_calendars().unwrap();
        let defaults: Vec<(&str, bool)> = calendars
            .iter()
            .map(|calendar| (calendar.id.as_str(), calendar.default))
            .collect();
        assert_eq!(defaults, [("me@example.org", true), ("other", false)]);
    }
}
