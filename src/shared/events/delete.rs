use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::{arg::CalendarIdArg, client::CalendarClient};

/// Delete a single event.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct EventDeleteCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// Stable event identifier (iCal `UID`).
    #[arg(value_name = "EVENT-ID")]
    pub event_id: String,
}

impl EventDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        client.delete_item(&calendar_id, &self.event_id)?;
        printer.out(Message::new("Event successfully deleted"))
    }
}
