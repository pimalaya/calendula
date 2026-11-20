//! # Event list
//!
//! The `calendula event list` command, printing the VEVENTs of a calendar
//! one line each.

use std::fmt;

use anyhow::Result;
use chrono::NaiveDate;
use clap::Parser;
use jiff::{Timestamp, civil::Date, tz::TimeZone};
use pimalaya_cli::{clap::args::PageTokenArg, listing::Listing, printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    capability::Capability,
    error::{self, ItemError},
    shared::{
        arg::CalendarIdArg,
        client::CalendarClient,
        event::Event,
        ical::IcalFamily,
        item::{CalendarItemQuery, CalendarTimeRange, list::listing_page},
    },
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
/// under a header naming the account, the calendar and where the page
/// stands. An item whose iCalendar does not parse prints no line but an
/// `Error:` line at the end, and the command exits 3.
///
/// JSON output: `{"events": [{"id", "calendarId", "etag", "uid",
/// "recurrenceId", "summary", "description", "location", "start", "end",
/// "allDay", "timeZone", "recurring", "status", "transparency",
/// "busyStatus", "organizer": {"email", "name"}, "attendees": [{"email",
/// "name", "partstat", "role", "rsvp", "cutype"}], "onlineMeetingUrl"}],
/// "nextPage"}`, `nextPage` being the number of the page following this
/// one as a string, which `--page-token` takes back, or `null` once the
/// listing is complete, and `errors` names each item it could not read,
/// and a listing the server cut short by a `null` id: `[{"id", "code",
/// "error", "sources"}]`.
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
    pub fn execute(self, printer: &mut impl Printer, client: &mut CalendarClient) -> Result<()> {
        client.require(&[Capability::ItemList])?;

        let page = match &self.page_token.token {
            Some(token) => Some(
                token
                    .parse::<u32>()
                    .map_err(|_| error::page_token_invalid(token))?,
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
        let (events, count, more, errors) = match &range {
            Some(range) => {
                let listed = client.list_items(&calendar_id, None, None, query)?;
                let mut errors = listed.errors;
                let (mut events, _) = Event::read_all(&listed.items, &mut errors, |item| {
                    Event::try_occurrences(item, Some(range))
                });
                events.sort_by_key(|event| event.start_secs);
                let more = page_window(&mut events, page, page_size);
                let count = events.len();
                (events, count, more, errors)
            }
            None => {
                let listed = client.list_items(&calendar_id, page, page_size, query)?;
                let mut errors = listed.errors;
                let (events, count) =
                    Event::read_all(&listed.items, &mut errors, Event::try_project);
                (events, count, listed.more, errors)
            }
        };

        let next_page = match page {
            Some(page) if more => Some((page + 1).to_string()),
            _ => None,
        };

        let mut listing = Listing::new(&client.name, Some(client.backend_name()));
        listing.head("Calendar", &calendar_id);
        let held = listing_page(page, page_size);
        listing.page(held, count, EVENT_NOUNS, next_page.as_deref());

        let output = EventListOutput {
            listing,
            zone: local_zone(),
            events,
            next_page,
            errors,
        };

        printer.out(&output)?;
        error::partial_when(&output.errors)
    }
}

/// Keeps page `page` of `size` of a window's occurrences, sorted by start
/// across the whole window, and answers whether another follows.
fn page_window(events: &mut Vec<Event>, page: Option<u32>, size: Option<u32>) -> bool {
    let (Some(page), Some(size)) = (page, size) else {
        return false;
    };
    let size = size as usize;
    let skip = (page as usize).saturating_sub(1).saturating_mul(size);

    events.drain(..skip.min(events.len()));
    let more = events.len() > size;
    events.truncate(size);
    more
}

/// What the header of an event listing calls one event, then several.
pub(crate) const EVENT_NOUNS: [&str; 2] = ["event", "events"];

/// The events of one page and the number of the next.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventListOutput {
    /// The header the events print under, its page placed.
    #[serde(skip)]
    pub listing: Listing<'static>,
    /// The zone a timed event's start and end print in.
    #[serde(skip)]
    pub zone: TimeZone,
    /// The listed events, in the order the backend returned them.
    pub events: Vec<Event>,
    /// The number of the page following this one, given back through
    /// `--page-token`, `null` once the listing is complete.
    pub next_page: Option<String>,
    /// The error of each item the page could not read, in the order met.
    pub errors: Vec<ItemError>,
}

impl fmt::Display for EventListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut listing = self.listing.clone();

        for event in &self.events {
            let (start, end) = row_times(event, &self.zone);

            listing.items.push([
                sanitize::text(&event.id).into_owned(),
                sanitize::text(&start).into_owned(),
                sanitize::text(&end).into_owned(),
                sanitize::text(event.summary.as_deref().unwrap_or_default()).into_owned(),
            ]);
        }

        for error in &self.errors {
            listing.errors.push(error);
        }

        write!(f, "{listing}")
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
            .start
            .clone()
            .unwrap_or_else(|| event.wire_start.clone());
        let first = start.parse::<Date>().ok();
        let end = match event.end.as_deref().map(str::parse::<Date>) {
            // NOTE: the JSON end is exclusive, the line's the last day,
            // never before the first: a zero-length whole day (DTEND equal
            // to DTSTART) ends on its start.
            Some(Ok(end)) => {
                let last = end.yesterday().unwrap_or(end);
                first.map_or(last, |first| last.max(first)).to_string()
            }
            _ => event.wire_end.clone(),
        };
        return (start, end);
    }

    let local = |secs: Option<i64>, raw: &str| {
        secs.and_then(|secs| Timestamp::from_second(secs).ok())
            .map(|at| {
                at.to_zoned_datetime(zone.clone())
                    .strftime("%Y-%m-%d %H:%M")
                    .to_string()
            })
            .unwrap_or_else(|| raw.to_string())
    };

    (
        local(event.start_secs, &event.wire_start),
        local(event.end_secs, &event.wire_end),
    )
}

#[cfg(test)]
mod tests {
    use jiff::tz::TimeZone;
    use pimalaya_cli::listing::{Listing, ListingPage};

    use crate::{
        error::ItemError,
        shared::{
            event::{
                Event,
                list::{EVENT_NOUNS, EventListOutput, row_times},
            },
            item::{CalendarItem, list::listing_page},
        },
    };

    fn event(all_day: bool, start: &str, end: &str) -> Event {
        Event {
            all_day,
            wire_start: start.into(),
            wire_end: end.into(),
            ..Default::default()
        }
    }

    /// A page of `count` events of the vdir account `perso`, paged by two
    /// unless `page` is `None`, `next` the page following it.
    fn listing(page: Option<u32>, count: usize, next: Option<&str>) -> Listing<'static> {
        let mut listing = Listing::new("perso", Some("vdir"));
        listing.head("Calendar", "personal");
        let held = listing_page(page, page.map(|_| 2));
        listing.page(held, count, EVENT_NOUNS, next);
        listing
    }

    /// The `errors` entry of an item `id` whose bytes do not parse, as the
    /// listing builds it.
    fn unparsed(id: &str) -> ItemError {
        let item = CalendarItem {
            id: id.into(),
            contents: b"not a calendar".to_vec(),
            ..Default::default()
        };
        let mut errors = Vec::new();
        Event::read_all(&[item], &mut errors, Event::try_project);
        errors.remove(0)
    }

    /// One aligned line per event: the id, the start and the end, then the
    /// summary, `-` standing for an empty one, under the account, the
    /// calendar and the page.
    #[test]
    fn events_print_one_line_each_under_a_header() {
        let mut standup = event(false, "20261030T080000Z", "20261030T083000Z");
        standup.id = "standup.ics".into();
        standup.summary = Some("Standup\x1b[2J".into());
        standup.start_secs = Some(1_793_347_200);
        standup.end_secs = Some(1_793_349_000);

        let mut holiday = event(true, "20260519", "20260521");
        holiday.id = "holiday.ics".into();
        holiday.start = Some("2026-05-19".into());
        holiday.end = Some("2026-05-21".into());

        let mut output = EventListOutput {
            listing: listing(Some(1), 2, Some("2")),
            zone: TimeZone::UTC,
            events: vec![standup, holiday],
            next_page: Some(String::from("2")),
            errors: Vec::new(),
        };

        let expected = "\
Account:   perso (vdir)
Calendar:  personal
Page:      1, events 1 to 2

standup.ics  2026-10-30 08:00  2026-10-30 08:30  Standup\u{fffd}[2J
holiday.ics  2026-05-19        2026-05-20        -

Next page: --page 2
";
        assert_eq!(output.to_string(), expected);

        output.listing = listing(Some(2), 1, None);
        output.events.truncate(1);
        output.next_page = None;

        let expected = "\
Account:   perso (vdir)
Calendar:  personal
Page:      2, event 3, the last

standup.ics  2026-10-30 08:00  2026-10-30 08:30  Standup\u{fffd}[2J
";
        assert_eq!(output.to_string(), expected);

        output.listing = listing(Some(3), 0, None);
        output.events.clear();

        let expected = "\
Account:   perso (vdir)
Calendar:  personal
Page:      3, empty
";
        assert_eq!(output.to_string(), expected);
    }

    /// The items the page could not read print last, after the rows and one
    /// blank line, one aligned `Error:` line each, and with no row after the
    /// header alone.
    #[test]
    fn the_errors_print_last_one_aligned_line_each() {
        let mut standup = event(false, "20261030T080000Z", "20261030T083000Z");
        standup.id = "standup.ics".into();
        standup.summary = Some("Standup".into());
        standup.start_secs = Some(1_793_347_200);
        standup.end_secs = Some(1_793_349_000);

        let mut output = EventListOutput {
            listing: listing(Some(1), 1, None),
            zone: TimeZone::UTC,
            events: vec![standup],
            next_page: None,
            errors: vec![unparsed("c"), unparsed("broken\x1b[2J")],
        };

        let expected = "\
Account:   perso (vdir)
Calendar:  personal
Page:      1, event 1, the last

standup.ics  2026-10-30 08:00  2026-10-30 08:30  Standup

Error:  c           item-unreadable  Cannot parse item `c` as iCalendar
Error:  broken\u{fffd}[2J  item-unreadable  Cannot parse item `broken\u{fffd}[2J` as iCalendar
";
        assert_eq!(output.to_string(), expected);

        output.listing = listing(Some(1), 0, None);
        output.events.clear();
        output.errors.truncate(1);

        let expected = "\
Account:   perso (vdir)
Calendar:  personal
Page:      1, empty

Error:  c  item-unreadable  Cannot parse item `c` as iCalendar
";
        assert_eq!(output.to_string(), expected);

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["errors"][0]["id"], "c");
        assert_eq!(json["errors"][0]["code"], "item-unreadable");
        assert_eq!(json["errors"][0]["sources"].as_array().unwrap().len(), 1);
    }

    /// A window lists every match unpaged, and `event search` names the UID
    /// it looked for.
    #[test]
    fn an_unpaged_listing_holds_all_its_events() {
        let mut found = event(true, "20260519", "20260520");
        found.id = "a.ics".into();
        found.summary = Some("Review".into());

        let mut listing = Listing::new("perso", Some("vdir"));
        listing.head("Calendar", "personal");
        listing.head("UID", "1@example.org");
        listing.page(ListingPage::All, 1, EVENT_NOUNS, None);

        let output = EventListOutput {
            listing,
            zone: TimeZone::UTC,
            events: vec![found],
            next_page: None,
            errors: Vec::new(),
        };

        let expected = "\
Account:   perso (vdir)
Calendar:  personal
UID:       1@example.org
Page:      all, event 1

a.ics  20260519  20260520  Review
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
        event.start = Some("2026-05-19".into());
        event.end = Some("2026-05-21".into());

        assert_eq!(
            row_times(&event, &TimeZone::UTC),
            ("2026-05-19".into(), "2026-05-20".into())
        );
    }

    #[test]
    fn a_one_day_event_starts_and_ends_on_it() {
        let mut event = event(true, "20260519", "20260520");
        event.start = Some("2026-05-19".into());
        event.end = Some("2026-05-20".into());

        assert_eq!(
            row_times(&event, &TimeZone::UTC),
            ("2026-05-19".into(), "2026-05-19".into())
        );
    }

    #[test]
    fn a_zero_length_whole_day_ends_on_its_start() {
        let mut event = event(true, "20260519", "20260519");
        event.start = Some("2026-05-19".into());
        event.end = Some("2026-05-19".into());

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
