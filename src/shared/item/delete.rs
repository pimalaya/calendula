//! # Item delete
//!
//! The `item delete` command, removing one iCalendar object from a
//! calendar.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::shared::{
    arg::CalendarIdArg,
    client::CalendarClient,
    receipt::{Write, WriteReceipt},
};

/// Delete a single iCalendar item.
///
/// JSON output: `{"id", "queued"}`.
#[derive(Debug, Parser)]
pub struct ItemDeleteCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// Item to delete, as `item list` reports it.
    #[arg(value_name = "ITEM-ID")]
    pub item_id: String,

    /// Gate the delete on this ETag, as a listing or a read reports it.
    ///
    /// The delete only lands if the backend still holds that version
    /// (RFC 9110 `If-Match`), which is how a concurrent write is caught.
    #[arg(long, value_name = "ETAG")]
    pub if_match: Option<String>,
}

impl ItemDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        client.delete_item(&calendar_id, &self.item_id, self.if_match.as_deref(), None)?;
        printer.out(WriteReceipt {
            id: self.item_id,
            queued: client.queues_writes(),
            what: "Item",
            write: Write::Delete,
        })
    }
}
