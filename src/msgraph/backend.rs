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
    client::{MsgraphClientStd, MsgraphClientStdConnectOptions},
    field::MsgraphField,
    rest::users::{
        calendars::{
            MsgraphCalendar,
            list::{MsgraphCalendarsListParams, MsgraphCalendarsListResponse},
        },
        events::{
            MsgraphEvent, MsgraphEventType,
            ical::{MSGRAPH_EVENT_ICAL_SELECT, MSGRAPH_EVENT_STASH_EXPAND},
            list::MsgraphEventsListParams,
        },
    },
    send::MsgraphSend,
};
use jiff::{ToSpan, civil::Date, tz::TimeZone};
use log::warn;
use pimalaya_config::secret::SecretResolver;
use pimalaya_stream::proxy::Proxy;
use secrecy::ExposeSecret;
use url::Url;

use crate::{
    config::{MsgraphConfig, default_msgraph_alpn},
    shared::{
        calendar::{Calendar, CalendarDiff},
        client::paginate,
        item::{CalendarItem, CalendarItemQuery},
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
            let url = Url::parse(&next).context("Cannot parse the calendar paging link")?;
            let coroutine =
                MsgraphSend::<MsgraphCalendarsListResponse>::get(&self.client.auth, url);
            page = self.client.run(coroutine)?.response;
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
    ) -> Result<Vec<CalendarItem>> {
        // NOTE: Graph models a VEVENT and nothing else, so any other kind
        // is answered without a round-trip.
        if query
            .kind
            .is_some_and(|kind| kind != IcalComponentKind::VEvent)
        {
            return Ok(Vec::new());
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
        for event in unique(events)? {
            // NOTE: a listing leaves out cancelledOccurrences, which Graph
            // returns only on a read of the master by id.
            let event = if event.event_type == Some(MsgraphEventType::SeriesMaster) {
                self.event(&event.id)?
            } else {
                event
            };
            let exceptions = self.exceptions(&event)?;
            let item = item_from(calendar_id, &event, &exceptions);
            if item.starts_within(query.range) {
                items.push(item);
            }
        }

        Ok(paginate(items, query.page, query.page_size))
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
            bail!("Item `{item_id}` changed on Microsoft Graph since it was read");
        }

        let exceptions = self.exceptions(&current)?;
        let exceptions: Vec<&MsgraphEvent> = exceptions.iter().collect();
        let base = current.to_ical_series(&exceptions);

        let patch = MsgraphEvent::update_from_ical(&contents, base.as_bytes())?;
        self.client.event_update(item_id, &patch)?;

        Ok(())
    }

    /// Deletes an event, the whole series for a master.
    pub fn delete_item(&mut self, _calendar_id: &str, item_id: &str) -> Result<()> {
        self.client.event_delete(item_id)?;
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
            )?
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
            .event_instances(&event.id, &start, &end, &params)?
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
    use super::*;

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
