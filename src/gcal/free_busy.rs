use std::fmt;

use anyhow::Result;
use chrono::NaiveDate;
use clap::Parser;
use io_gcal::v3::rest::freebusy::{GcalFreeBusyRequest, GcalFreeBusyRequestItem};
use pimalaya_cli::{
    printer::Printer,
    table::{Cell, Row, Table, TableStyle},
};
use serde::Serialize;

use crate::{
    gcal::{client::GcalClient, render},
    shared::items::CalendarTimeRange,
};

/// Query when calendars are busy over a window.
///
/// Availability is a question, not a resource: no component family can
/// carry it, which is why it lives here. Pass one or more calendar ids;
/// with none, the account's default calendar is queried.
///
/// JSON output: `{"calendars": [{"id", "busy": [{"start", "end"}],
/// "errors"}]}`.
#[derive(Debug, Parser)]
pub struct GcalFreeBusyCommand {
    /// The calendars to query. Repeat the flag for several; falls back
    /// to `calendar.default`.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR-ID")]
    pub calendar_ids: Vec<String>,

    /// Start of the window (inclusive, YYYY-MM-DD).
    #[arg(long, value_name = "DATE")]
    pub from: Option<NaiveDate>,

    /// End of the window (inclusive, YYYY-MM-DD).
    #[arg(long, value_name = "DATE")]
    pub to: Option<NaiveDate>,
}

impl GcalFreeBusyCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GcalClient) -> Result<()> {
        let style = client.account.table_style();

        let calendar_ids = match self.calendar_ids.is_empty() {
            false => self.calendar_ids,
            true => vec![client.account.calendar_id(None)?],
        };

        let range = CalendarTimeRange::from_days(self.from, self.to)?.unwrap_or_default();
        let request = GcalFreeBusyRequest {
            time_min: range.start.as_deref().map(render::rfc3339),
            time_max: range.end.as_deref().map(render::rfc3339),
            items: calendar_ids
                .into_iter()
                .map(|id| GcalFreeBusyRequestItem { id: Some(id) })
                .collect(),
            ..Default::default()
        };

        let response = client.free_busy_query(&request)?.response;

        let rows = response
            .calendars
            .into_iter()
            .map(|(id, calendar)| CalendarBusy {
                id,
                busy: calendar
                    .busy
                    .into_iter()
                    .map(|period| BusyPeriod {
                        start: period.start.unwrap_or_default(),
                        end: period.end.unwrap_or_default(),
                    })
                    .collect(),
                errors: calendar
                    .errors
                    .into_iter()
                    .filter_map(|error| error.reason)
                    .collect(),
            })
            .collect();

        printer.out(FreeBusyTable { style, rows })
    }
}

/// The rendered free/busy answer.
#[derive(Clone, Debug, Serialize)]
pub struct FreeBusyTable {
    #[serde(skip)]
    pub style: TableStyle,
    #[serde(rename = "calendars")]
    pub rows: Vec<CalendarBusy>,
}

/// The busy periods of one queried calendar.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct CalendarBusy {
    /// The calendar the periods belong to.
    pub id: String,
    /// The periods during which it is busy.
    pub busy: Vec<BusyPeriod>,
    /// Why the answer is incomplete, when it is.
    pub errors: Vec<String>,
}

/// One busy period, as the RFC 3339 bounds the API reports.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct BusyPeriod {
    /// The start of the period.
    pub start: String,
    /// The end of the period.
    pub end: String,
}

impl fmt::Display for FreeBusyTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table.load_style(self.style).set_header(Row::from([
            Cell::new("CALENDAR"),
            Cell::new("BUSY FROM"),
            Cell::new("BUSY UNTIL"),
        ]));

        for calendar in &self.rows {
            if calendar.busy.is_empty() {
                let note = match calendar.errors.is_empty() {
                    true => String::from("free"),
                    false => calendar.errors.join(", "),
                };

                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(&calendar.id))
                    .add_cell(Cell::new(note))
                    .add_cell(Cell::new(""));
                table.add_row(row);
                continue;
            }

            for period in &calendar.busy {
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(&calendar.id))
                    .add_cell(Cell::new(&period.start))
                    .add_cell(Cell::new(&period.end));
                table.add_row(row);
            }
        }

        writeln!(f)?;
        writeln!(f, "{table}")
    }
}
