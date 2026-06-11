use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::caldav::client::CaldavClient;

/// Delete a CalDAV calendar by id.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct CaldavCalendarDeleteCommand {
    /// Calendar identifier (last path segment of the calendar URL).
    #[arg(value_name = "ID")]
    pub id: String,
}

impl CaldavCalendarDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CaldavClient) -> Result<()> {
        client.delete_calendar(&self.id)?;

        let msg = format!("Calendar `{}` successfully deleted", self.id);
        printer.out(Message::new(msg))
    }
}
