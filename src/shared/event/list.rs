//! # Event list
//!
//! The `calendula event list` command, rendering the VEVENTs of a
//! calendar as a table.

use std::fmt;

use anyhow::Result;
use chrono::NaiveDate;
use clap::Parser;
use jiff::{Timestamp, civil::Date, tz::TimeZone};
use pimalaya_cli::{
    printer::Printer,
    table::{Cell, Color, ColumnConstraint, ContentArrangement, Row, Table, TableStyle},
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::{
    arg::CalendarIdArg,
    client::CalendarClient,
    event::Event,
    ical::IcalFamily,
    item::{CalendarItemQuery, CalendarTimeRange},
};

/// List the events of a calendar.
///
/// Only VEVENT components are rendered; the other kinds a calendar
/// holds (VTODO, VJOURNAL) are dropped, so use `item list` for the
/// unfiltered raw view.
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
/// (`TZ`, then the system's). The table shows them in that local zone
/// (`2026-10-30 09:00`), a whole-day event by its days, the end being
/// the last one (`2026-05-19`).
///
/// JSON output: `{"events": [{"id", "uid", "recurrenceId", "summary",
/// "description", "location", "start", "end", "allDay", "startsAt",
/// "endsAt", "timeZone", "recurring", "status", "transparency",
/// "busyStatus", "organizer": {"email", "name"}, "attendees": [{"email",
/// "name", "partstat", "role", "rsvp", "cutype"}], "onlineMeetingUrl"}]}`.
#[derive(Debug, Parser)]
pub struct EventListCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// 1-indexed page number. Defaults to 1.
    #[arg(short, long, value_name = "N")]
    pub page: Option<u32>,

    /// Number of items per page.
    #[arg(short = 's', long, value_name = "N")]
    pub page_size: Option<u32>,

    /// Only list events on or after this day (inclusive, YYYY-MM-DD).
    #[arg(long, value_name = "DATE")]
    pub from: Option<NaiveDate>,

    /// Only list events on or before this day (inclusive, YYYY-MM-DD).
    #[arg(long, value_name = "DATE")]
    pub to: Option<NaiveDate>,

    /// Maximum width of the rendered table, in terminal columns.
    #[arg(long = "max-width", short = 'w', value_name = "COLUMNS")]
    pub max_width: Option<u16>,
}

impl EventListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
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

        let items = client.list_items(
            &calendar_id,
            CalendarItemQuery {
                page: self.page,
                page_size,
                range: range.as_ref(),
                kind: IcalFamily::Event.kind(),
                ..Default::default()
            },
        )?;
        let events = match &range {
            Some(range) => {
                let mut events: Vec<Event> = items
                    .iter()
                    .flat_map(|item| Event::occurrences(item, Some(range)))
                    .collect();
                events.sort_by_key(|event| event.start_secs);
                events
            }
            None => items.iter().flat_map(Event::project).collect(),
        };

        printer.out(EventListOutput::new(&client, self.max_width, events))
    }
}

/// The per-column colors an event listing renders with.
#[derive(Clone, Copy, Debug)]
struct EventColors {
    id: Color,
    summary: Color,
    start: Color,
    end: Color,
}

/// The rendered event listing.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventListOutput {
    /// The table preset the account configures.
    #[serde(skip)]
    pub style: TableStyle,
    /// How the table spreads its columns over the terminal.
    #[serde(skip)]
    pub arrangement: ContentArrangement,
    /// The `--max-width` cap, when one was given.
    #[serde(skip)]
    pub max_width: Option<u16>,
    #[serde(skip)]
    colors: EventColors,
    /// The listed events, in the order the backend returned them.
    pub events: Vec<Event>,
}

impl EventListOutput {
    /// A listing of `events`, rendered as the account configures.
    pub fn new(client: &CalendarClient, max_width: Option<u16>, events: Vec<Event>) -> Self {
        Self {
            style: client.account.table_style(),
            arrangement: client.account.table_arrangement(),
            max_width,
            colors: EventColors {
                id: client.account.events_list_table_id_color(),
                summary: client.account.events_list_table_summary_color(),
                start: client.account.events_list_table_start_color(),
                end: client.account.events_list_table_end_color(),
            },
            events,
        }
    }
}

impl fmt::Display for EventListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let zone = TimeZone::try_system().unwrap_or(TimeZone::UTC);
        let mut table = Table::new();

        table
            .load_style(self.style)
            .set_content_arrangement(self.arrangement.clone())
            .set_header(Row::from(vec![
                Cell::new("ID"),
                Cell::new("SUMMARY"),
                Cell::new("START"),
                Cell::new("END"),
            ]))
            .add_rows(self.events.iter().map(|event| {
                let mut row = Row::new();
                row.max_height(1);
                row.add_cell(Cell::new(&event.id).fg(self.colors.id));
                row.add_cell(Cell::new(&event.summary).fg(self.colors.summary));
                let (start, end) = table_times(event, &zone);
                row.add_cell(Cell::new(start).fg(self.colors.start));
                row.add_cell(Cell::new(end).fg(self.colors.end));
                row
            }));

        // NOTE: a time cut short (`2026-10-1…`) no longer says when, so
        // squeezing the table into the terminal truncates the id and the
        // summary but not these two.
        for index in [2, 3] {
            if let Some(column) = table.column_mut(index) {
                column.set_constraint(ColumnConstraint::ContentWidth);
            }
        }

        if let Some(width) = self.max_width {
            table.set_width(width);
        }

        writeln!(f)?;
        writeln!(f, "{table}")
    }
}

/// The start and end an event's row shows, for a person to read: a
/// timed event at its instants in `zone`, a whole-day one by its days,
/// the end inclusive. An event whose times did not resolve shows them
/// as iCalendar spells them.
fn table_times(event: &Event, zone: &TimeZone) -> (String, String) {
    if event.all_day {
        let start = event
            .starts_at
            .clone()
            .unwrap_or_else(|| event.start.clone());
        let first = start.parse::<Date>().ok();
        let end = match event.ends_at.as_deref().map(str::parse::<Date>) {
            // NOTE: the JSON end is exclusive, the table's the last day,
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
    use super::*;

    fn event(all_day: bool, start: &str, end: &str) -> Event {
        Event {
            all_day,
            start: start.into(),
            end: end.into(),
            ..Default::default()
        }
    }

    #[test]
    fn a_timed_event_shows_in_the_local_zone() {
        let mut event = event(false, "20261030T080000Z", "20261030T093000Z");
        event.start_secs = Some(1_793_347_200);
        event.end_secs = Some(1_793_352_600);
        let zone = TimeZone::get("Europe/Madrid").unwrap();

        assert_eq!(
            table_times(&event, &zone),
            ("2026-10-30 09:00".into(), "2026-10-30 10:30".into())
        );
    }

    #[test]
    fn a_whole_day_event_ends_on_its_last_day() {
        let mut event = event(true, "20260519", "20260521");
        event.starts_at = Some("2026-05-19".into());
        event.ends_at = Some("2026-05-21".into());

        assert_eq!(
            table_times(&event, &TimeZone::UTC),
            ("2026-05-19".into(), "2026-05-20".into())
        );
    }

    #[test]
    fn a_one_day_event_starts_and_ends_on_it() {
        let mut event = event(true, "20260519", "20260520");
        event.starts_at = Some("2026-05-19".into());
        event.ends_at = Some("2026-05-20".into());

        assert_eq!(
            table_times(&event, &TimeZone::UTC),
            ("2026-05-19".into(), "2026-05-19".into())
        );
    }

    #[test]
    fn a_zero_length_whole_day_ends_on_its_start() {
        let mut event = event(true, "20260519", "20260519");
        event.starts_at = Some("2026-05-19".into());
        event.ends_at = Some("2026-05-19".into());

        assert_eq!(
            table_times(&event, &TimeZone::UTC),
            ("2026-05-19".into(), "2026-05-19".into())
        );
    }

    #[test]
    fn unresolved_times_show_as_written() {
        let event = event(false, "20261030T090000", "20261030T100000");

        assert_eq!(
            table_times(&event, &TimeZone::UTC),
            ("20261030T090000".into(), "20261030T100000".into())
        );
    }
}
