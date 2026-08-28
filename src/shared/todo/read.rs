use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::{arg::CalendarIdArg, client::CalendarClient};

/// Read a single todo (raw iCalendar bytes).
///
/// JSON output: `{"message": "..."}`, carrying the raw iCalendar.
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
        printer.out(Message::new(contents))
    }
}
