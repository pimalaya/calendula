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
    clap::args::PageTokenArg,
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
/// printing a `nextPage` was truncated, and that cursor goes back through
/// `--page-token` to drain the rest of the round.
///
/// JSON output: `{"events": [<raw Graph delta>...], "nextPage",
/// "@odata.deltaLink"}`, `nextPage` Graph's `@odata.nextLink`, `null` on
/// the round's last page, which carries the `@odata.deltaLink`.
#[derive(Debug, Parser)]
pub struct MsgraphEventDeltaCommand {
    /// Calendar id; omit for the default calendar.
    #[arg(
        short = 'k',
        long = "calendar",
        value_name = "CALENDAR-ID",
        conflicts_with_all = ["delta_link", "page_token"]
    )]
    pub calendar: Option<String>,
    /// First day of the window the round covers (inclusive).
    #[arg(
        long,
        value_name = "DATE",
        required_unless_present_any = ["delta_link", "page_token"],
        conflicts_with_all = ["delta_link", "page_token"]
    )]
    pub from: Option<NaiveDate>,
    /// Last day of the window the round covers (inclusive).
    #[arg(
        long,
        value_name = "DATE",
        required_unless_present_any = ["delta_link", "page_token"],
        conflicts_with_all = ["delta_link", "page_token"]
    )]
    pub to: Option<NaiveDate>,
    /// Open the next round from the last page of a previous one, instead
    /// of a fresh round.
    ///
    /// Takes the `@odata.deltaLink` that page returned. The link already
    /// carries the calendar and the window, so neither is passed again.
    #[arg(long, value_name = "URL", conflicts_with = "page_token")]
    pub delta_link: Option<String>,
    /// The page of a round to resume at, carrying the calendar and the
    /// window.
    #[command(flatten)]
    pub page_token: PageTokenArg,
}

impl MsgraphEventDeltaCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let link = self.page_token.token.as_ref().or(self.delta_link.as_ref());

        let page = match (link, self.from, self.to) {
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
            _ => bail!("Give `--from` and `--to`, a `--delta-link` or a `--page-token`"),
        }
        .response;

        printer.out(MsgraphEventDeltaOutput {
            events: page.value,
            next_page: page.next_link,
            delta_link: page.delta_link,
        })
    }
}

/// A page of the events delta, with the cursor and the link continuing
/// the round.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphEventDeltaOutput {
    /// The raw Graph event deltas of this page.
    #[serde(rename = "events")]
    #[schemars(with = "Vec<serde_json::Value>")]
    pub events: Vec<MsgraphEventDelta>,
    /// Graph's `@odata.nextLink` draining the rest of the round, `null` on
    /// its last page.
    pub next_page: Option<String>,
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

        if !records.items.is_empty() && (self.delta_link.is_some() || self.next_page.is_some()) {
            writeln!(f)?;
        }

        if let Some(delta_link) = &self.delta_link {
            writeln!(f, "delta-link: {}", sanitize::text(delta_link))?;
        }
        if let Some(cursor) = &self.next_page {
            writeln!(f, "Next page: {}", sanitize::text(cursor))?;
        }
        Ok(())
    }
}
