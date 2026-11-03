//! # Journal delete
//!
//! The `journal delete` command, removing one VJOURNAL entry from a
//! calendar.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::shared::{
    arg::CalendarIdArg,
    client::CalendarClient,
    receipt::{Write, WriteReceipt},
};

/// Delete a single journal entry.
///
/// JSON output: `{"id", "queued"}`.
#[derive(Debug, Parser)]
pub struct JournalDeleteCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// Journal entry to delete, as `journal list` reports it.
    #[arg(value_name = "JOURNAL-ID")]
    pub journal_id: String,

    /// Gate the delete on this ETag, as `item list` or `item read` reports it.
    ///
    /// The delete only lands if the backend still holds that version
    /// (RFC 9110 `If-Match`), which is how a concurrent write is caught.
    #[arg(long, value_name = "ETAG")]
    pub if_match: Option<String>,
}

impl JournalDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        client.delete_item(
            &calendar_id,
            &self.journal_id,
            self.if_match.as_deref(),
            None,
        )?;
        printer.out(WriteReceipt {
            id: self.journal_id,
            queued: client.queues_writes(),
            what: "Journal entry",
            write: Write::Delete,
        })
    }
}
