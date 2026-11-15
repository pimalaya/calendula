//! # Instances
//!
//! The `calendula gcal events instances` command, expanding a recurring
//! event into its occurrences (`events.instances`).

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
use serde_json::{Map, Value};

use crate::{
    gcal::{
        client::GcalClient,
        render::{self, boundary_of, str_of},
    },
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
/// JSON output: `{"instances": [<event>...]}`, every instance of every
/// page as Google sent it (`id`, `status`, `summary`, `start`, `end`,
/// `originalStartTime` and the rest).
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

        let params = GcalEventInstancesParams {
            time_min: time_min.as_deref(),
            time_max: time_max.as_deref(),
            show_deleted: self.show_deleted,
            ..Default::default()
        };
        let segments = [
            "calendars",
            &calendar_id,
            "events",
            &self.event_id,
            "instances",
        ];
        let instances = client.raw_items(&segments, &params)?;

        let mut listing = Listing::new(&client.name, Some("gcal"));
        listing.head("Calendar", &calendar_id);
        listing.head("Event", &self.event_id);
        let nouns = ["instance", "instances"];
        listing.page(ListingPage::All, instances.len(), nouns, None);

        printer.out(GcalInstancesOutput { listing, instances })
    }
}

/// The instance listing.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GcalInstancesOutput {
    /// The header the instances print under, its page placed.
    #[serde(skip)]
    pub listing: Listing<'static>,
    /// The occurrences of the series, each as Google sent it.
    #[schemars(with = "Vec<Map<String, Value>>")]
    pub instances: Vec<Value>,
}

impl fmt::Display for GcalInstancesOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut listing = self.listing.clone();

        for instance in &self.instances {
            listing.items.push([
                sanitize::text(str_of(instance, "id")).into_owned(),
                sanitize::text(str_of(instance, "status")).into_owned(),
                sanitize::text(boundary_of(instance, "start")).into_owned(),
                sanitize::text(boundary_of(instance, "end")).into_owned(),
                sanitize::text(str_of(instance, "summary")).into_owned(),
            ]);
        }

        write!(f, "{listing}")
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// The instances print as Google sent them, the time pairs nested as
    /// Google nests them, the text reading a timestamp or a date.
    #[test]
    fn the_instances_print_as_google_sent_them() {
        let timed = json!({
            "kind": "calendar#event",
            "id": "e1_20261112T090000Z",
            "status": "confirmed",
            "summary": "Standup",
            "start": {"dateTime": "2026-11-12T10:00:00+01:00", "timeZone": "Europe/Paris"},
            "end": {"dateTime": "2026-11-12T10:15:00+01:00", "timeZone": "Europe/Paris"},
            "originalStartTime": {"dateTime": "2026-11-12T10:00:00+01:00"},
            "recurringEventId": "e1",
        });
        let whole = json!({
            "id": "e2_20261113",
            "status": "cancelled",
            "start": {"date": "2026-11-13"},
            "end": {"date": "2026-11-14"},
        });
        let output = GcalInstancesOutput {
            listing: Listing::new("perso", Some("gcal")),
            instances: vec![timed.clone(), whole.clone()],
        };

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json, json!({"instances": [timed, whole]}));

        let text = output.to_string();
        assert!(text.contains("2026-11-12T10:00:00+01:00"), "{text}");
        assert!(text.contains("2026-11-13"), "{text}");
        assert!(text.contains("cancelled"), "{text}");
    }
}
