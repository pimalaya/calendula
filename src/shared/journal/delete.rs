use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::{arg::CalendarIdArg, client::CalendarClient};

/// Delete a single journal entry.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct JournalDeleteCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// Stable journal entry identifier.
    #[arg(value_name = "JOURNAL-ID")]
    pub journal_id: String,
}

impl JournalDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        client.delete_item(&calendar_id, &self.journal_id)?;
        printer.out(Message::new("Journal entry successfully deleted"))
    }
}
