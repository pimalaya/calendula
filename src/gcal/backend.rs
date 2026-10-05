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

use anyhow::{Context, Result, bail};
use ical::component::IcalComponentKind;
use io_gcal::v3::{
    client::GcalClientStd,
    rest::{
        calendar_list::{GcalCalendarListEntry, list::GcalCalendarListListParams},
        calendars::GcalCalendar,
        events::{
            GcalEvent, import::GcalEventImportParams, insert::GcalEventInsertParams,
            list::GcalEventsListParams, update::GcalEventUpdateParams,
        },
    },
};
use log::warn;
use pimalaya_config::secret::SecretResolver;

use crate::{
    config::GcalConfig,
    gcal::{client::connect, render::rfc3339},
    shared::{
        calendar::{Calendar, CalendarDiff},
        client::paginate,
        item::{CalendarItem, CalendarItemQuery},
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
    /// shared API returns it whole.
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
        id: &str,
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

        if created_id != id {
            warn!("google assigned the calendar id `{created_id}`, not `{id}`");
        }

        if let Some(color) = color {
            self.paint(&created_id, color)?;
        }

        Ok(created_id)
    }

    /// Applies a partial update: title and description patch the calendar,
    /// colour patches the calendar list entry.
    pub fn update_calendar(&mut self, id: &str, patch: CalendarDiff) -> Result<()> {
        if patch.name.is_some() || patch.description.is_some() {
            let calendar = GcalCalendar {
                summary: patch.name,
                description: patch.description.flatten(),
                ..Default::default()
            };

            self.client.calendar_patch(id, &calendar)?;
        }

        if let Some(color) = patch.color {
            match color {
                Some(color) => self.paint(id, &color)?,
                None => bail!("Google calendars always carry a colour; set one instead"),
            }
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
    /// soon as the requested window is covered, so a page the caller never
    /// reaches is never fetched.
    pub fn list_items(
        &mut self,
        calendar_id: &str,
        query: CalendarItemQuery<'_>,
    ) -> Result<Vec<CalendarItem>> {
        // NOTE: Google models a VEVENT and nothing else, so any other
        // kind is answered without a round-trip rather than by filtering
        // a listing that could never hold one.
        if query
            .kind
            .is_some_and(|kind| kind != IcalComponentKind::VEvent)
        {
            return Ok(Vec::new());
        }

        let range = query.range;
        let time_min = range.and_then(|range| range.start.as_deref()).map(rfc3339);
        let time_max = range.and_then(|range| range.end.as_deref()).map(rfc3339);
        let wanted = window(query.page, query.page_size);

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

            if wanted.is_some_and(|wanted| events.len() >= wanted) {
                break;
            }

            match current.next_page_token {
                Some(next) => page_token = Some(next),
                None => break,
            }
        }

        Ok(paginate(
            resources(calendar_id, &events),
            query.page,
            query.page_size,
        ))
    }

    /// Reads one event, with the exceptions of a series folded into it.
    pub fn get_item(&mut self, calendar_id: &str, item_id: &str) -> Result<CalendarItem> {
        let event = self
            .client
            .event_get(calendar_id, item_id, None, None)
            .with_context(|| format!("Read item `{item_id}` from calendar `{calendar_id}`"))?
            .response;

        let exceptions = self.exceptions(calendar_id, &event)?;

        Ok(item_from(
            calendar_id,
            &event,
            &exceptions.iter().collect::<Vec<_>>(),
        ))
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
    /// A document carrying a UID is imported, so the UID survives as the
    /// event's `iCalUID` and the resource keeps the identity it already
    /// had; one carrying none is inserted, and Google mints both ids itself.
    pub fn create_item(&mut self, calendar_id: &str, contents: Vec<u8>) -> Result<String> {
        let event = GcalEvent::from_ical(&contents)?;

        let created = match event.ical_uid.as_deref().filter(|uid| !uid.is_empty()) {
            Some(_) => {
                let params = GcalEventImportParams::default();
                self.client.event_import(calendar_id, &event, &params)?
            }
            None => {
                let params = GcalEventInsertParams::default();
                self.client.event_insert(calendar_id, &event, &params)?
            }
        };

        Ok(created.response.id.unwrap_or_default())
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
    ) -> Result<()> {
        let projected = GcalEvent::from_ical(&contents)?;
        let current = self
            .client
            .event_get(calendar_id, item_id, None, None)?
            .response;

        let event = projected.merge(&current);
        let params = GcalEventUpdateParams::default();

        self.client
            .event_update(calendar_id, item_id, &event, &params, if_match)?;

        Ok(())
    }

    /// Deletes an event.
    ///
    /// Google offers no precondition on a delete, so an `if_match` is
    /// refused rather than dropped.
    pub fn delete_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        if_match: Option<&str>,
    ) -> Result<()> {
        if if_match.is_some() {
            bail!("Google Calendar cannot gate a delete on an ETag");
        }
        self.client.event_delete(calendar_id, item_id, None, None)?;
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

/// How many items a listing has to hold before the requested window can be
/// cut out of it, or `None` when the caller asked for everything.
fn window(page: Option<u32>, page_size: Option<u32>) -> Option<usize> {
    let size = page_size?;
    let page = page.unwrap_or(1).max(1);

    Some((page as usize).saturating_mul(size as usize))
}

/// Projects a calendar list entry onto the shared [`Calendar`], preferring
/// the title this user gave the calendar over its own.
fn calendar_from(entry: GcalCalendarListEntry) -> Calendar {
    let id = entry.id.unwrap_or_default();

    Calendar {
        name: entry
            .summary_override
            .or(entry.summary)
            .unwrap_or_else(|| id.clone()),
        id,
        description: entry.description,
        color: entry.background_color,
        default: false,
    }
}

/// Folds a batch of Google events into calendar object resources.
///
/// An exception is not an item of its own: RFC 4791 4.1 keeps components
/// sharing a UID in one resource. One whose master this batch did not
/// return still files alone, since dropping it would lose a listed item.
fn resources(calendar_id: &str, events: &[GcalEvent]) -> Vec<CalendarItem> {
    let masters: Vec<&str> = events
        .iter()
        .filter(|event| event.recurring_event_id.is_none())
        .filter_map(|event| event.id.as_deref())
        .collect();

    let folded = |event: &GcalEvent| {
        event
            .recurring_event_id
            .as_deref()
            .is_some_and(|master| masters.contains(&master))
    };

    events
        .iter()
        .filter(|event| !folded(event))
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
    use super::*;
    use io_gcal::v3::rest::events::GcalEventDateTime;

    use crate::shared::event::Event;

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
        assert_eq!(events[0].summary, "Stand-up");
        assert_eq!(events[0].start, "20260814T090000Z");
        assert_eq!(events[0].end, "20260814T100000Z");
    }

    #[test]
    fn the_window_covers_every_page_up_to_the_requested_one() {
        assert_eq!(window(None, None), None);
        assert_eq!(window(None, Some(25)), Some(25));
        assert_eq!(window(Some(3), Some(25)), Some(75));

        // NOTE: a page of zero clamps to the first page, as the shared
        // pagination clamps it.
        assert_eq!(window(Some(0), Some(25)), Some(25));
    }
}
