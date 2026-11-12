//! # Item read
//!
//! The `item read` command, printing one iCalendar object as stored.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    capability::Capability,
    shared::{arg::CalendarIdArg, client::CalendarClient},
};

/// Read a single iCalendar item.
///
/// The iCalendar prints as stored, CRLF read as LF and any character that
/// could drive the terminal replaced.
///
/// JSON output: `{"contents", "etag"}`, the raw iCalendar, its bytes
/// kept, and the version a write can be gated on (`--if-match`), `null`
/// when the backend has none.
#[derive(Debug, Parser)]
pub struct ItemReadCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// Item to read, as `item list` reports it.
    #[arg(value_name = "ITEM-ID")]
    pub item_id: String,
}

impl ItemReadCommand {
    pub fn execute(self, printer: &mut impl Printer, client: &mut CalendarClient) -> Result<()> {
        client.require(&[Capability::ItemRead])?;

        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        let item = client.get_item(&calendar_id, &self.item_id)?;
        let contents = String::from_utf8_lossy(&item.contents).into_owned();
        printer.out(ItemReadOutput {
            contents,
            etag: item.etag,
        })
    }
}

/// The read item, as the calendar stores it.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemReadOutput {
    /// The raw iCalendar bytes, lossily decoded as UTF-8.
    pub contents: String,
    /// The entity tag of the version read, when the backend has one.
    pub etag: Option<String>,
}

impl fmt::Display for ItemReadOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", printable(&self.contents).trim_end())
    }
}

/// Defuses an iCalendar for a terminal, keeping its tabs and line breaks.
pub fn printable(contents: &str) -> String {
    let contents = contents.replace("\r\n", "\n");

    contents
        .chars()
        .map(|c| match c {
            '\t' | '\n' => c,
            '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}' => char::REPLACEMENT_CHARACTER,
            c if c.is_control() => char::REPLACEMENT_CHARACTER,
            c => c,
        })
        .collect()
}
