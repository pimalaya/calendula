use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::{arg::CalendarIdArg, client::CalendarClient};

/// Read a single event (raw iCalendar bytes).
///
/// JSON output: `{"message": "..."}`, carrying the raw iCalendar.
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
        printer.out(Message::new(contents))
    }
}
