use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::{arg::CalendarIdArg, client::CalendarClient};

/// Delete a single iCalendar item.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct ItemDeleteCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// Stable item identifier (iCal `UID`).
    #[arg(value_name = "ITEM-ID")]
    pub item_id: String,
}

impl ItemDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        client.delete_item(&calendar_id, &self.item_id)?;
        printer.out(Message::new("Item successfully deleted"))
    }
}
