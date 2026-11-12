//! # Event list
//!
//! The `calendula event list` command, printing the VEVENTs of a calendar
//! one line each.

use std::fmt;

use anyhow::{Context, Result};
use chrono::NaiveDate;
use clap::Parser;
use jiff::{Timestamp, civil::Date, tz::TimeZone};
use pimalaya_cli::{clap::args::PageTokenArg, columns::Columns, printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::{
    arg::CalendarIdArg,
    client::CalendarClient,
    event::Event,
    ical::IcalFamily,
    item::{CalendarItemQuery, CalendarTimeRange},
    listing::Listing,
};

/// List the events of a calendar.
///
/// Only VEVENT components are listed; the other kinds a calendar holds
/// (VTODO, VJOURNAL) are dropped, so use `item list` for the unfiltered
/// raw view.
///
/// Pass `--from` and `--to` (YYYY-MM-DD, both inclusive) to narrow the
/// listing to a window: CalDAV runs it server-side, the local backends
/// after parsing. A window lifts the default page-size cap, so every
/// match is returned.
///
/// A window also expands recurring events: each occurrence overlapping
/// it lists on its own, at its own times, with the `recurrenceId` that
/// addresses it beside the item `id`, overrides applied and excluded
/// dates dropped, in start order. The bounds are UTC midnights, and one
/// open on its end reaches a year past its start.
///
/// Without a window the listing is the raw one: items in the order the
/// backend returns them, a series once at its first start. For what is
/// coming, give `--from` (or draw `event agenda`).
///
/// Times resolve through their `TZID`, the calendar's `VTIMEZONE` or
/// the time-zone database; a floating time is read in the local zone
/// (`TZ`, then the system's). Each line shows the id, the start and the
/// end in that local zone (`2026-10-30 09:00`), a whole-day event by its
/// days, the end being the last one (`2026-05-19`), then the summary,
/// between a header naming the account and the calendar and a footer
/// saying where the page stands.
///
/// JSON output: `{"events": [{"id", "uid", "recurrenceId", "summary",
/// "description", "location", "start", "end", "allDay", "startsAt",
/// "endsAt", "timeZone", "recurring", "status", "transparency",
/// "busyStatus", "organizer": {"email", "name"}, "attendees": [{"email",
/// "name", "partstat", "role", "rsvp", "cutype"}], "onlineMeetingUrl"}],
/// "nextPage"}`, `nextPage` being the number of the page following this
/// one as a string, which `--page-token` takes back, or `null` once the
/// listing is complete.
#[derive(Debug, Parser)]
pub struct EventListCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// 1-indexed page number. Defaults to 1.
    #[arg(short, long, value_name = "N")]
    pub page: Option<u32>,

    /// Number of items per page, zero listing every one.
    #[arg(short = 's', long, value_name = "N")]
    pub page_size: Option<u32>,

    #[command(flatten)]
    pub page_token: PageTokenArg,

    /// Only list events on or after this day (inclusive, YYYY-MM-DD).
    #[arg(long, value_name = "DATE")]
    pub from: Option<NaiveDate>,

    /// Only list events on or before this day (inclusive, YYYY-MM-DD).
    #[arg(long, value_name = "DATE")]
    pub to: Option<NaiveDate>,
}

impl EventListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let page = match &self.page_token.token {
            Some(token) => Some(
                token
                    .parse::<u32>()
                    .with_context(|| format!("Invalid page token `{token}`"))?,
            ),
            None => self.page,
        };

        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        let range = CalendarTimeRange::from_days(self.from, self.to)?;

        // NOTE: a window must return every match, so the default
        // page-size cap only applies to the unfiltered listing.
        let page_size = match range {
            Some(_) => self.page_size,
            None => self
                .page_size
                .or(Some(client.account.events_list_page_size())),
        };
        let page_size = page_size.filter(|size| *size > 0);
        let page = page_size.map(|_| page.unwrap_or(1).max(1));

        let query = CalendarItemQuery {
            range: range.as_ref(),
            kind: IcalFamily::Event.kind(),
            ..Default::default()
        };
        let listed = client.list_items(&calendar_id, page, page_size, query)?;
        let events = match &range {
            Some(range) => {
                let mut events: Vec<Event> = listed
                    .items
                    .iter()
                    .flat_map(|item| Event::occurrences(item, Some(range)))
                    .collect();
                events.sort_by_key(|event| event.start_secs);
                events
            }
            None => listed.items.iter().flat_map(Event::project).collect(),
        };

        let mut listing = Listing::new(&client.name, client.backend_name(), EVENT_NOUNS);
        listing.push("Calendar", &calendar_id);
        listing.page = page;
        listing.page_size = page_size;
        listing.count = listed.items.len();

        let next_page = match page {
            Some(page) if listed.more => Some((page + 1).to_string()),
            _ => None,
        };

        printer.out(EventListOutput {
            listing,
            zone: local_zone(),
            events,
            next_page,
        })
    }
}

/// What the footer of an event listing calls one event, then several.
pub(crate) const EVENT_NOUNS: [&str; 2] = ["event", "events"];

/// The events of one page and the number of the next.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventListOutput {
    /// The header and footer the events print between.
    #[serde(skip)]
    pub listing: Listing,
    /// The zone a timed event's start and end print in.
    #[serde(skip)]
    pub zone: TimeZone,
    /// The listed events, in the order the backend returned them.
    pub events: Vec<Event>,
    /// The number of the page following this one, given back through
    /// `--page-token`, `null` once the listing is complete.
    pub next_page: Option<String>,
}

impl fmt::Display for EventListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut columns = Columns::default();

        for event in &self.events {
            let (start, end) = row_times(event, &self.zone);

            columns.push([
                sanitize::text(&event.id).into_owned(),
                sanitize::text(&start).into_owned(),
                sanitize::text(&end).into_owned(),
                sanitize::text(&event.summary).into_owned(),
            ]);
        }

        self.listing
            .write(f, &columns.to_string(), self.next_page.as_deref())
    }
}

/// The local zone a floating time is read in and a timed event prints
/// in: `TZ`, then the system's, UTC when neither resolves.
pub(crate) fn local_zone() -> TimeZone {
    TimeZone::try_system().unwrap_or(TimeZone::UTC)
}

/// The start and end an event's row shows, for a person to read: a
/// timed event at its instants in `zone`, a whole-day one by its days,
/// the end inclusive. An event whose times did not resolve shows them
/// as iCalendar spells them.
pub(crate) fn row_times(event: &Event, zone: &TimeZone) -> (String, String) {
    if event.all_day {
        let start = event
            .starts_at
            .clone()
            .unwrap_or_else(|| event.start.clone());
        let first = start.parse::<Date>().ok();
        let end = match event.ends_at.as_deref().map(str::parse::<Date>) {
            // NOTE: the JSON end is exclusive, the line's the last day,
            // never before the first: a zero-length whole day (DTEND equal
            // to DTSTART) ends on its start.
            Some(Ok(end)) => {
                let last = end.yesterday().unwrap_or(end);
                first.map_or(last, |first| last.max(first)).to_string()
            }
            _ => event.end.clone(),
        };
        return (start, end);
    }

    let local = |secs: Option<i64>, raw: &str| {
        secs.and_then(|secs| Timestamp::from_second(secs).ok())
            .map(|at| {
                at.to_zoned(zone.clone())
                    .strftime("%Y-%m-%d %H:%M")
                    .to_string()
            })
            .unwrap_or_else(|| raw.to_string())
    };

    (
        local(event.start_secs, &event.start),
        local(event.end_secs, &event.end),
    )
}

#[cfg(test)]
mod tests {
    use jiff::tz::TimeZone;

    use crate::shared::{
        event::{
            Event,
            list::{EVENT_NOUNS, EventListOutput, row_times},
        },
        listing::Listing,
    };

    fn event(all_day: bool, start: &str, end: &str) -> Event {
        Event {
            all_day,
            start: start.into(),
            end: end.into(),
            ..Default::default()
        }
    }

    /// A page of `count` events of the vdir account `perso`, paged by two
    /// unless `page` is `None`.
    fn listing(page: Option<u32>, count: usize) -> Listing {
        let mut listing = Listing::new("perso", "vdir", EVENT_NOUNS);
        listing.push("Calendar", "personal");
        listing.page = page;
        listing.page_size = page.map(|_| 2);
        listing.count = count;
        listing
    }

    /// One aligned line per event: the id, the start and the end, then the
    /// summary, `-` standing for an empty one, between the account and
    /// calendar above and the page under.
    #[test]
    fn events_print_one_line_each_between_a_header_and_a_footer() {
        let mut standup = event(false, "20261030T080000Z", "20261030T083000Z");
        standup.id = "standup.ics".into();
        standup.summary = "Standup\x1b[2J".into();
        standup.start_secs = Some(1_793_347_200);
        standup.end_secs = Some(1_793_349_000);

        let mut holiday = event(true, "20260519", "20260521");
        holiday.id = "holiday.ics".into();
        holiday.starts_at = Some("2026-05-19".into());
        holiday.ends_at = Some("2026-05-21".into());

        let mut output = EventListOutput {
            listing: listing(Some(1), 2),
            zone: TimeZone::UTC,
            events: vec![standup, holiday],
            next_page: Some(String::from("2")),
        };

        let expected = "\
Account:    perso (vdir)
Calendar:   personal

standup.ics  2026-10-30 08:00  2026-10-30 08:30  Standup\u{fffd}[2J
holiday.ics  2026-05-19        2026-05-20        -

Page:       1, events 1 to 2
Next page:  --page 2
";
        assert_eq!(output.to_string(), expected);

        output.listing = listing(Some(2), 1);
        output.events.truncate(1);
        output.next_page = None;

        let expected = "\
Account:    perso (vdir)
Calendar:   personal

standup.ics  2026-10-30 08:00  2026-10-30 08:30  Standup\u{fffd}[2J

Page:       2, event 3, the last
";
        assert_eq!(output.to_string(), expected);

        output.listing = listing(Some(3), 0);
        output.events.clear();

        let expected = "\
Account:    perso (vdir)
Calendar:   personal

Page:       3, empty
";
        assert_eq!(output.to_string(), expected);
    }

    /// A window lists every match unpaged, and `event find` names the UID
    /// it looked for.
    #[test]
    fn an_unpaged_listing_holds_all_its_events() {
        let mut found = event(true, "20260519", "20260520");
        found.id = "a.ics".into();
        found.summary = "Review".into();

        let mut listing = listing(None, 1);
        listing.push("UID", "1@example.org");

        let output = EventListOutput {
            listing,
            zone: TimeZone::UTC,
            events: vec![found],
            next_page: None,
        };

        let expected = "\
Account:    perso (vdir)
Calendar:   personal
UID:        1@example.org

a.ics  20260519  20260520  Review

Page:       all, event 1
";
        assert_eq!(output.to_string(), expected);
    }

    #[test]
    fn a_timed_event_shows_in_the_local_zone() {
        let mut event = event(false, "20261030T080000Z", "20261030T093000Z");
        event.start_secs = Some(1_793_347_200);
        event.end_secs = Some(1_793_352_600);
        let zone = TimeZone::get("Europe/Madrid").unwrap();

        assert_eq!(
            row_times(&event, &zone),
            ("2026-10-30 09:00".into(), "2026-10-30 10:30".into())
        );
    }

    #[test]
    fn a_whole_day_event_ends_on_its_last_day() {
        let mut event = event(true, "20260519", "20260521");
        event.starts_at = Some("2026-05-19".into());
        event.ends_at = Some("2026-05-21".into());

        assert_eq!(
            row_times(&event, &TimeZone::UTC),
            ("2026-05-19".into(), "2026-05-20".into())
        );
    }

    #[test]
    fn a_one_day_event_starts_and_ends_on_it() {
        let mut event = event(true, "20260519", "20260520");
        event.starts_at = Some("2026-05-19".into());
        event.ends_at = Some("2026-05-20".into());

        assert_eq!(
            row_times(&event, &TimeZone::UTC),
            ("2026-05-19".into(), "2026-05-19".into())
        );
    }

    #[test]
    fn a_zero_length_whole_day_ends_on_its_start() {
        let mut event = event(true, "20260519", "20260519");
        event.starts_at = Some("2026-05-19".into());
        event.ends_at = Some("2026-05-19".into());

        assert_eq!(
            row_times(&event, &TimeZone::UTC),
            ("2026-05-19".into(), "2026-05-19".into())
        );
    }

    #[test]
    fn unresolved_times_show_as_written() {
        let event = event(false, "20261030T090000", "20261030T100000");

        assert_eq!(
            row_times(&event, &TimeZone::UTC),
            ("20261030T090000".into(), "20261030T100000".into())
        );
    }
}
