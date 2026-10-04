//! # Todo delete
//!
//! The `calendula todo delete` command, removing one VTODO from the
//! selected calendar.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::{arg::CalendarIdArg, client::CalendarClient};

/// Delete a single todo.
///
/// Immediate and unconditional: the task is removed with no
/// confirmation prompt and no copy kept.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct TodoDeleteCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// Todo to delete, as `todo list` reports it.
    #[arg(value_name = "TODO-ID")]
    pub todo_id: String,

    /// Gate the delete on this ETag, as `item list` or `item read` reports it.
    ///
    /// The delete only lands if the backend still holds that version
    /// (RFC 9110 `If-Match`), which is how a concurrent write is caught.
    #[arg(long, value_name = "ETAG")]
    pub if_match: Option<String>,
}

impl TodoDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        client.delete_item(&calendar_id, &self.todo_id, self.if_match.as_deref())?;
        printer.out(Message::new("Todo successfully deleted"))
    }
}
