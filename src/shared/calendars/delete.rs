use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::client::CalendarClient;

/// Delete the given calendar and every item it contains.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct CalendarDeleteCommand {
    /// Calendar to delete. Mandatory: unlike the other shared-API
    /// commands, deletion never falls back to the `calendar.default`
    /// config.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR-ID")]
    pub id: String,
}

impl CalendarDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        client.delete_calendar(&self.id)?;

        let msg = format!("Calendar `{}` successfully deleted", self.id);
        printer.out(Message::new(msg))
    }
}
