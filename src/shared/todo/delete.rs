use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::{arg::CalendarIdArg, client::CalendarClient};

/// Delete a single todo.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct TodoDeleteCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// Stable todo identifier.
    #[arg(value_name = "TODO-ID")]
    pub todo_id: String,
}

impl TodoDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        client.delete_item(&calendar_id, &self.todo_id)?;
        printer.out(Message::new("Todo successfully deleted"))
    }
}
