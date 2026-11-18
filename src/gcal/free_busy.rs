//! # Free/busy
//!
//! The `calendula gcal free-busy` command, querying when calendars are busy
//! over a window.

use std::fmt;

use anyhow::{Result, bail};
use chrono::NaiveDate;
use clap::Parser;
use io_gcal::v3::rest::freebusy::{GcalError, GcalFreeBusyRequest, GcalFreeBusyRequestItem};
use pimalaya_cli::{
    fields::Fields,
    printer::Printer,
    records::{Record, Records},
    sanitize,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    gcal::{client::GcalClient, render},
    shared::item::CalendarTimeRange,
};

/// Query when calendars are busy over a window.
///
/// Availability is a question, not a resource: no component family carries
/// it, which is why it lives here. Pass one or more calendar ids; with
/// none, the account's default calendar is queried.
///
/// JSON output: `{"calendars": [{"id", "busy": [{"start", "end"}],
/// "queryErrors": [{"domain", "reason"}]}]}`, `queryErrors` being the
/// error objects the API answered for the calendar, raw.
#[derive(Debug, Parser)]
pub struct GcalFreeBusyCommand {
    /// The calendars to query. Repeat the flag for several; falls back to
    /// `calendar.default`.
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
        let calendar_ids = match self.calendar_ids.is_empty() {
            false => self.calendar_ids,
            true => vec![client.account.calendar_id(None)?],
        };

        if self.from.is_none() || self.to.is_none() {
            bail!("gcal free-busy needs a window: give both --from and --to");
        }

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
                query_errors: calendar.errors,
            })
            .collect();

        printer.out(GcalFreeBusyOutput { rows })
    }
}

/// The free/busy answer.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GcalFreeBusyOutput {
    /// The queried calendars, one entry each.
    #[serde(rename = "calendars")]
    pub rows: Vec<CalendarBusy>,
}

/// The busy periods of one queried calendar.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CalendarBusy {
    /// The calendar the periods belong to.
    pub id: String,
    /// The periods during which it is busy.
    pub busy: Vec<BusyPeriod>,
    /// Why the answer is incomplete, when it is: the error objects the
    /// API answered for the calendar (`domain`, `reason`), raw. Not the
    /// item errors of a listing's `errors`.
    #[schemars(with = "Vec<serde_json::Map<String, serde_json::Value>>")]
    pub query_errors: Vec<GcalError>,
}

/// One busy period, as the RFC 3339 bounds the API reports.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BusyPeriod {
    /// The start of the period.
    pub start: String,
    /// The end of the period.
    pub end: String,
}

impl fmt::Display for GcalFreeBusyOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut records = Records::default();

        for calendar in &self.rows {
            let mut fields = Fields::default();

            let periods = calendar.busy.iter().map(|period| {
                let (start, end) = (&period.start, &period.end);
                sanitize::text(&format!("{start} to {end}")).into_owned()
            });
            let periods: Vec<String> = periods.collect();

            if !periods.is_empty() {
                fields.push("Busy", periods.join("\n"));
            }

            let reasons: Vec<&str> = calendar
                .query_errors
                .iter()
                .filter_map(|error| error.reason.as_deref())
                .collect();

            if !reasons.is_empty() {
                fields.push("Errors", sanitize::text(&reasons.join(", ")));
            }

            let state = match (periods.is_empty(), calendar.query_errors.is_empty()) {
                (false, _) => vec![String::from("busy")],
                (true, true) => vec![String::from("free")],
                (true, false) => Vec::new(),
            };

            records.push(Record {
                title: sanitize::text(&calendar.id).into_owned(),
                state,
                fields,
                body: String::new(),
            });
        }

        write!(f, "{records}")
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// The API's error objects print raw under `queryErrors`, so `errors`
    /// keeps the one meaning it has across calendula, the item errors of
    /// a listing.
    #[test]
    fn the_api_errors_print_raw_under_their_own_key() {
        let output = GcalFreeBusyOutput {
            rows: vec![CalendarBusy {
                id: "nope@example.org".into(),
                busy: Vec::new(),
                query_errors: vec![GcalError {
                    domain: Some("global".into()),
                    reason: Some("notFound".into()),
                }],
            }],
        };

        assert_eq!(
            serde_json::to_value(&output).unwrap(),
            json!({
                "calendars": [{
                    "id": "nope@example.org",
                    "busy": [],
                    "queryErrors": [{"domain": "global", "reason": "notFound"}],
                }],
            })
        );
        assert_eq!(output.to_string(), "nope@example.org\nErrors:  notFound\n");
    }
}
