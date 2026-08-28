use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::{arg::CalendarIdArg, client::CalendarClient};

/// Read a single iCalendar item.
///
/// The raw iCalendar bytes are printed verbatim on stdout.
///
/// JSON output: `{"message": "..."}`, carrying the raw iCalendar.
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
        printer.out(Message::new(contents))
    }
}
