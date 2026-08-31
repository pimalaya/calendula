//! # Todo read
//!
//! The `calendula todo read` command, printing one VTODO as the
//! calendar stores it.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::{arg::CalendarIdArg, client::CalendarClient};

/// Read a single todo (raw iCalendar bytes).
///
/// JSON output: `{"contents"}`, carrying the raw iCalendar.
#[derive(Debug, Parser)]
pub struct TodoReadCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// Stable todo identifier.
    #[arg(value_name = "TODO-ID")]
    pub todo_id: String,
}

impl TodoReadCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        let item = client.get_item(&calendar_id, &self.todo_id)?;
        let contents = String::from_utf8_lossy(&item.contents).into_owned();
        printer.out(TodoReadOutput { contents })
    }
}

/// The read todo, as the calendar stores it.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TodoReadOutput {
    /// The raw iCalendar bytes, lossily decoded as UTF-8.
    pub contents: String,
}

impl fmt::Display for TodoReadOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.contents.trim_end())
    }
}
