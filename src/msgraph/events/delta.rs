//! # Event delta command
//!
//! Runs the Graph incremental sync over a calendar view and renders one
//! page of changes.

use core::fmt;

use anyhow::{Result, bail};
use chrono::NaiveDate;
use clap::Parser;
use io_msgraph::v1::rest::users::events::delta::MsgraphEventDelta;
use pimalaya_cli::{
    fields::Fields,
    printer::Printer,
    records::{Record, Records},
    sanitize,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::msgraph::{
    client::{MsgraphClient, calendar_refusal},
    events::render::event_fields,
};

/// List one page of the events changed or removed since the last round
/// (the Graph incremental sync of a calendar view).
///
/// Without `--delta-link` this opens a fresh round over the days `--from`
/// to `--to` (inclusive), enumerating every occurrence in them and ending
/// with an `@odata.deltaLink`. Feed that link back through `--delta-link`
/// for the next round, which returns only what changed since. A page
/// ending with an `@odata.nextLink` was truncated, and that link goes back
/// through the same flag to drain the rest.
///
/// JSON output: `{"events": [<raw Graph delta>...], "@odata.nextLink",
/// "@odata.deltaLink"}`.
#[derive(Debug, Parser)]
pub struct MsgraphEventDeltaCommand {
    /// Calendar id; omit for the default calendar.
    #[arg(
        short = 'k',
        long = "calendar",
        value_name = "CALENDAR-ID",
        conflicts_with = "delta_link"
    )]
    pub calendar: Option<String>,
    /// First day of the window the round covers (inclusive).
    #[arg(
        long,
        value_name = "DATE",
        required_unless_present = "delta_link",
        conflicts_with = "delta_link"
    )]
    pub from: Option<NaiveDate>,
    /// Last day of the window the round covers (inclusive).
    #[arg(
        long,
        value_name = "DATE",
        required_unless_present = "delta_link",
        conflicts_with = "delta_link"
    )]
    pub to: Option<NaiveDate>,
    /// Continue a round from a previous page instead of opening a new one.
    ///
    /// Takes the `@odata.deltaLink` or `@odata.nextLink` that page
    /// returned. The link already carries the calendar and the window, so
    /// neither is passed again.
    #[arg(long, value_name = "URL")]
    pub delta_link: Option<String>,
}

impl MsgraphEventDeltaCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let page = match (&self.delta_link, self.from, self.to) {
            (Some(link), _, _) => client.events_delta_from_link(link)?,
            (None, Some(from), Some(to)) => {
                if to < from {
                    bail!("Invalid time range: `--to` {to} is before `--from` {from}");
                }

                let start = format!("{}T00:00:00Z", from.format("%Y-%m-%d"));
                let end = to.succ_opt().unwrap_or(to);
                let end = format!("{}T00:00:00Z", end.format("%Y-%m-%d"));

                client
                    .events_delta(self.calendar.as_deref(), &start, &end)
                    .map_err(calendar_refusal)?
            }
            _ => bail!("Give `--from` and `--to`, or a `--delta-link`"),
        }
        .response;

        printer.out(MsgraphEventDeltaOutput {
            events: page.value,
            next_link: page.next_link,
            delta_link: page.delta_link,
        })
    }
}

/// A page of the events delta, with the links continuing the round.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphEventDeltaOutput {
    /// The raw Graph event deltas of this page.
    #[serde(rename = "events")]
    #[schemars(with = "Vec<serde_json::Value>")]
    pub events: Vec<MsgraphEventDelta>,
    /// The link draining the rest of the page, when it was truncated.
    #[serde(rename = "@odata.nextLink", skip_serializing_if = "Option::is_none")]
    pub next_link: Option<String>,
    /// The link opening the next round, when the round ended here.
    #[serde(rename = "@odata.deltaLink", skip_serializing_if = "Option::is_none")]
    pub delta_link: Option<String>,
}

impl fmt::Display for MsgraphEventDeltaOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut records = Records::default();

        for delta in &self.events {
            let status = match delta.removed {
                Some(_) => "removed",
                None => "changed",
            };

            // NOTE: a removed event carries its id alone.
            let fields = match delta.removed {
                Some(_) => Fields::default(),
                None => event_fields(&delta.event),
            };

            records.push(Record {
                title: sanitize::text(&delta.event.id).into_owned(),
                state: vec![String::from(status)],
                fields,
                body: String::new(),
            });
        }

        write!(f, "{records}")?;

        if !records.items.is_empty() && (self.delta_link.is_some() || self.next_link.is_some()) {
            writeln!(f)?;
        }

        if let Some(delta_link) = &self.delta_link {
            writeln!(f, "delta-link: {}", sanitize::text(delta_link))?;
        }
        if let Some(next_link) = &self.next_link {
            writeln!(f, "next-link: {}", sanitize::text(next_link))?;
            writeln!(f, "(page truncated: pass the next-link to --delta-link)")?;
        }
        Ok(())
    }
}
