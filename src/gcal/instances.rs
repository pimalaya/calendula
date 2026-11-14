//! # Instances
//!
//! The `calendula gcal instances` command, expanding a recurring event into
//! its occurrences.

use std::fmt;

use anyhow::Result;
use chrono::NaiveDate;
use clap::Parser;
use io_gcal::v3::rest::events::instances::GcalEventInstancesParams;
use pimalaya_cli::{
    listing::{Listing, ListingPage},
    printer::Printer,
    sanitize,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    gcal::{client::GcalClient, render},
    shared::item::CalendarTimeRange,
};

/// Expand a recurring event into its occurrences.
///
/// The shared listing returns the series rather than its instances, since
/// the series is what round-trips through the projection. This asks the
/// server to expand it, the only way to see the moved and cancelled
/// occurrences a rule alone does not describe.
///
/// Each line shows the instance id, its status, its start and end as the
/// server spells them, then its summary, under a header naming the
/// account, the calendar and the series and counting them.
///
/// JSON output: `{"instances": [{"id", "summary", "start", "end",
/// "status", "originalStart"}]}`.
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

            rows.extend(page.items.into_iter().map(|event| {
                InstanceRow {
                    id: event.id.unwrap_or_default(),
                    summary: event.summary.unwrap_or_default(),
                    start: render::boundary(event.start.as_ref()),
                    end: render::boundary(event.end.as_ref()),
                    status: event
                        .status
                        .as_ref()
                        .map(|status| render::event_status(status).to_owned())
                        .unwrap_or_default(),
                    original_start: render::boundary(event.original_start_time.as_ref()),
                }
            }));

            match page.next_page_token {
                Some(next) => page_token = Some(next),
                None => break,
            }
        }

        let mut listing = Listing::new(&client.name, Some("gcal"));
        listing.head("Calendar", &calendar_id);
        listing.head("Event", &self.event_id);
        let nouns = ["instance", "instances"];
        listing.page(ListingPage::All, rows.len(), nouns, None);

        printer.out(GcalInstancesOutput { listing, rows })
    }
}

/// The instance listing.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GcalInstancesOutput {
    /// The header the instances print under, its page placed.
    #[serde(skip)]
    pub listing: Listing<'static>,
    /// The occurrences of the series, one entry each.
    #[serde(rename = "instances")]
    pub rows: Vec<InstanceRow>,
}

/// One occurrence of a recurring series.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InstanceRow {
    /// The instance identifier, which addresses this occurrence alone.
    pub id: String,
    /// The title, inherited from the series unless overridden.
    pub summary: String,
    /// The start of the occurrence.
    pub start: String,
    /// The end of the occurrence.
    pub end: String,
    /// `confirmed`, `tentative` or `cancelled`.
    pub status: String,
    /// Where the recurrence rule placed it, set once it has been moved.
    pub original_start: String,
}

impl fmt::Display for GcalInstancesOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut listing = self.listing.clone();

        for instance in &self.rows {
            listing.items.push([
                sanitize::text(&instance.id).into_owned(),
                sanitize::text(&instance.status).into_owned(),
                sanitize::text(&instance.start).into_owned(),
                sanitize::text(&instance.end).into_owned(),
                sanitize::text(&instance.summary).into_owned(),
            ]);
        }

        write!(f, "{listing}")
    }
}
