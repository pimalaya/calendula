//! # Event read
//!
//! The `calendula event read` command, printing one VEVENT as the
//! calendar stores it.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::{arg::CalendarIdArg, client::CalendarClient};

/// Read a single event (raw iCalendar bytes).
///
/// JSON output: `{"contents"}`, carrying the raw iCalendar.
#[derive(Debug, Parser)]
pub struct EventReadCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// Stable event identifier (iCal `UID`).
    #[arg(value_name = "EVENT-ID")]
    pub event_id: String,
}

impl EventReadCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        let item = client.get_item(&calendar_id, &self.event_id)?;
        let contents = String::from_utf8_lossy(&item.contents).into_owned();
        printer.out(EventReadOutput { contents })
    }
}

/// The read event, as the calendar stores it.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventReadOutput {
    /// The raw iCalendar bytes, lossily decoded as UTF-8.
    pub contents: String,
}

impl fmt::Display for EventReadOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.contents.trim_end())
    }
}
