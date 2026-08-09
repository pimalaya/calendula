use std::fmt;

use anyhow::Result;
use chrono::NaiveDate;
use clap::Parser;
use io_gcal::v3::rest::events::instances::GcalEventInstancesParams;
use pimalaya_cli::{
    printer::Printer,
    table::{Cell, Row, Table, TableStyle},
};
use serde::Serialize;

use crate::{
    gcal::{client::GcalClient, render},
    shared::items::CalendarTimeRange,
};

/// Expand a recurring event into its occurrences.
///
/// The shared listing returns the series rather than its instances,
/// since the series is what round-trips through the projection. This
/// asks the server to expand it, which is the only way to see the
/// moved and cancelled occurrences a rule alone does not describe.
///
/// JSON output: `{"instances": [{"id", "summary", "start", "end",
/// "status", "original-start"}]}`.
#[derive(Debug, Parser)]
pub struct GcalInstancesCommand {
    /// The calendar holding the series. Falls back to
    /// `calendar.default`.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR-ID")]
    pub calendar_id: Option<String>,

    /// The recurring event to expand.
    #[arg(value_name = "EVENT-ID")]
    pub event_id: String,

    /// Only return instances on or after this day (inclusive).
    #[arg(long, value_name = "DATE")]
    pub from: Option<NaiveDate>,

    /// Only return instances on or before this day (inclusive).
    #[arg(long, value_name = "DATE")]
    pub to: Option<NaiveDate>,

    /// Include the cancelled occurrences of the series.
    #[arg(long)]
    pub show_deleted: bool,
}

impl GcalInstancesCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GcalClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar_id)?;
        let style = client.account.table_style();

        let range = CalendarTimeRange::from_days(self.from, self.to)?.unwrap_or_default();
        let time_min = range.start.as_deref().map(render::rfc3339);
        let time_max = range.end.as_deref().map(render::rfc3339);

        let mut rows = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let params = GcalEventInstancesParams {
                time_min: time_min.as_deref(),
                time_max: time_max.as_deref(),
                page_token: page_token.as_deref(),
                show_deleted: self.show_deleted,
                ..Default::default()
            };
            let page = client
                .event_instances(&calendar_id, &self.event_id, &params)?
                .response;

            rows.extend(page.items.into_iter().map(|event| InstanceRow {
                id: event.id.unwrap_or_default(),
                summary: event.summary.unwrap_or_default(),
                start: render::boundary(event.start.as_ref()),
                end: render::boundary(event.end.as_ref()),
                status: event.status.map(render::event_status).unwrap_or(""),
                original_start: render::boundary(event.original_start_time.as_ref()),
            }));

            match page.next_page_token {
                Some(next) => page_token = Some(next),
                None => break,
            }
        }

        printer.out(InstancesTable { style, rows })
    }
}

/// The rendered instance listing.
#[derive(Clone, Debug, Serialize)]
pub struct InstancesTable {
    #[serde(skip)]
    pub style: TableStyle,
    #[serde(rename = "instances")]
    pub rows: Vec<InstanceRow>,
}

/// One occurrence of a recurring series.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct InstanceRow {
    /// The instance identifier, which addresses this occurrence alone.
    pub id: String,
    /// The title, inherited from the series unless the occurrence
    /// overrides it.
    pub summary: String,
    /// The start of the occurrence.
    pub start: String,
    /// The end of the occurrence.
    pub end: String,
    /// `confirmed`, `tentative` or `cancelled`.
    pub status: &'static str,
    /// Where the recurrence rule placed the occurrence, set when it has
    /// since been moved.
    pub original_start: String,
}

impl fmt::Display for InstancesTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(self.style)
            .set_header(Row::from([
                Cell::new("ID"),
                Cell::new("SUMMARY"),
                Cell::new("START"),
                Cell::new("END"),
                Cell::new("STATUS"),
            ]))
            .add_rows(self.rows.iter().map(|instance| {
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(&instance.id))
                    .add_cell(Cell::new(&instance.summary))
                    .add_cell(Cell::new(&instance.start))
                    .add_cell(Cell::new(&instance.end))
                    .add_cell(Cell::new(instance.status));
                row
            }));

        writeln!(f)?;
        writeln!(f, "{table}")
    }
}
