//! Google Calendar adapter for the shared cross-protocol client.
//!
//! Projects [`io_gcal`]'s Calendar API v3 resources onto calendula's own
//! shared types over a connected [`GcalClientStd`]. A calendar is a
//! calendar list entry and an item is an event of it; item ids are the
//! event ids the API returned, verbatim.
//!
//! Google splits a calendar across two resources: the calendar itself
//! carries the title and the description, while the per-user calendar
//! list entry carries the colour, so an update touches whichever of the
//! two the patch names.
//!
//! Because the server owns the query, a [`CalendarTimeRange`] is pushed
//! down as the `timeMin` and `timeMax` parameters of `events.list`
//! rather than applied locally, and the listing walks `nextPageToken`
//! only as far as the requested window reaches.

use anyhow::{Context, Result, bail};
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

use crate::{
    config::GcalConfig,
    gcal::{client::connect, project, render::rfc3339},
    shared::{
        calendars::{Calendar, CalendarDiff},
        client::paginate,
        items::{CalendarItem, CalendarTimeRange},
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
    pub fn new(config: GcalConfig) -> Result<Self> {
        Ok(Self {
            client: connect(&config)?,
        })
    }

    /// Lists every calendar of the user's calendar list, walking the
    /// pagination to the end since a calendar list is small and the
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

    /// Creates a secondary calendar, then paints it through the
    /// calendar list entry when a colour is asked for, since Google
    /// keeps the colour on the per-user entry rather than on the
    /// calendar itself.
    ///
    /// Google mints the id of a calendar it creates, so the requested
    /// `id` cannot be honoured; the assigned one is returned instead,
    /// which is what the caller reports.
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

    /// Applies a partial update: the title and the description patch
    /// the calendar, the colour patches the calendar list entry.
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

    /// Deletes a secondary calendar and every event it holds. Google
    /// refuses this on a primary calendar, which surfaces as its own
    /// error.
    pub fn delete_calendar(&mut self, id: &str) -> Result<()> {
        self.client.calendar_delete(id)?;
        Ok(())
    }

    /// Lists the events of a calendar, each projected onto an iCalendar
    /// document, narrowing the query server-side when a range is given.
    ///
    /// The pagination stops as soon as the requested window is covered,
    /// so a page the caller never reaches is never fetched.
    pub fn list_items(
        &mut self,
        calendar_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
        range: Option<&CalendarTimeRange>,
    ) -> Result<Vec<CalendarItem>> {
        let time_min = range.and_then(|range| range.start.as_deref()).map(rfc3339);
        let time_max = range.and_then(|range| range.end.as_deref()).map(rfc3339);
        let wanted = window(page, page_size);

        let mut items = Vec::new();
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

            items.extend(
                current
                    .items
                    .into_iter()
                    .map(|event| item_from(calendar_id, event)),
            );

            if wanted.is_some_and(|wanted| items.len() >= wanted) {
                break;
            }

            match current.next_page_token {
                Some(next) => page_token = Some(next),
                None => break,
            }
        }

        Ok(paginate(items, page, page_size))
    }

    /// Reads one event, projected onto an iCalendar document.
    pub fn get_item(&mut self, calendar_id: &str, item_id: &str) -> Result<CalendarItem> {
        let event = self
            .client
            .event_get(calendar_id, item_id, None, None)
            .with_context(|| format!("Read item `{item_id}` from calendar `{calendar_id}`"))?
            .response;

        Ok(item_from(calendar_id, event))
    }

    /// Creates an event from an iCalendar document.
    ///
    /// A document carrying a UID is imported, so the UID survives as
    /// the event's `iCalUID` and the resource keeps the identity it
    /// already had; one carrying none is inserted, and Google mints
    /// both ids itself.
    pub fn create_item(&mut self, calendar_id: &str, contents: Vec<u8>) -> Result<String> {
        let event = project::to_event(&contents)?;

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

    /// Replaces an event from an iCalendar document, optionally gated
    /// on `if_match`.
    ///
    /// The current server event is read first and serves as the base
    /// the projection merges onto, so the fields no iCalendar property
    /// models survive a write that would otherwise clear them.
    pub fn update_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<()> {
        let projected = project::to_event(&contents)?;
        let current = self
            .client
            .event_get(calendar_id, item_id, None, None)?
            .response;

        let event = project::merge(&current, projected);
        let params = GcalEventUpdateParams::default();

        self.client
            .event_update(calendar_id, item_id, &event, &params, if_match)?;

        Ok(())
    }

    /// Deletes an event unconditionally.
    pub fn delete_item(&mut self, calendar_id: &str, item_id: &str) -> Result<()> {
        self.client.event_delete(calendar_id, item_id, None, None)?;
        Ok(())
    }

    /// Sets the background colour of a calendar through its calendar
    /// list entry, the only resource carrying one.
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

/// How many items a listing has to hold before the requested window can
/// be cut out of it, or `None` when the caller asked for everything.
fn window(page: Option<u32>, page_size: Option<u32>) -> Option<usize> {
    let size = page_size?;
    let page = page.unwrap_or(1).max(1);

    Some((page as usize).saturating_mul(size as usize))
}

/// Projects a Google calendar list entry onto the shared [`Calendar`],
/// preferring the title this user gave the calendar over its own.
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
    }
}

/// Projects a Google event onto the shared [`CalendarItem`], its
/// contents synthesized by the projection.
fn item_from(calendar_id: &str, event: GcalEvent) -> CalendarItem {
    CalendarItem {
        id: event.id.clone().unwrap_or_default(),
        calendar_id: calendar_id.to_owned(),
        etag: event.etag.clone(),
        contents: project::to_ical(&event).into_bytes(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_window_covers_every_page_up_to_the_requested_one() {
        assert_eq!(window(None, None), None);
        assert_eq!(window(None, Some(25)), Some(25));
        assert_eq!(window(Some(3), Some(25)), Some(75));

        // A page of zero is clamped to the first page, as the shared
        // pagination clamps it.
        assert_eq!(window(Some(0), Some(25)), Some(25));
    }
}
