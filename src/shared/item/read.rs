//! # Item read
//!
//! The `item read` command, printing one iCalendar object verbatim.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::{arg::CalendarIdArg, client::CalendarClient};

/// Read a single iCalendar item.
///
/// The raw iCalendar bytes are printed verbatim on stdout.
///
/// JSON output: `{"contents"}`, carrying the raw iCalendar.
#[derive(Debug, Parser)]
pub struct ItemReadCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// Stable item identifier (iCal `UID`).
    #[arg(value_name = "ITEM-ID")]
    pub item_id: String,
}

impl ItemReadCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        let item = client.get_item(&calendar_id, &self.item_id)?;
        let contents = String::from_utf8_lossy(&item.contents).into_owned();
        printer.out(ItemReadOutput { contents })
    }
}

/// The read item, as the calendar stores it.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemReadOutput {
    /// The raw iCalendar bytes, lossily decoded as UTF-8.
    pub contents: String,
}

impl fmt::Display for ItemReadOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.contents.trim_end())
    }
}
