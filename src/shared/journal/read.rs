use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::{arg::CalendarIdArg, client::CalendarClient};

/// Read a single journal entry (raw iCalendar bytes).
///
/// JSON output: `{"message": "..."}`, carrying the raw iCalendar.
#[derive(Debug, Parser)]
pub struct JournalReadCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// Stable journal entry identifier.
    #[arg(value_name = "JOURNAL-ID")]
    pub journal_id: String,
}

impl JournalReadCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        let item = client.get_item(&calendar_id, &self.journal_id)?;
        let contents = String::from_utf8_lossy(&item.contents).into_owned();
        printer.out(Message::new(contents))
    }
}
