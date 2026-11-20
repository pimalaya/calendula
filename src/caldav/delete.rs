//! # DELETE command
//!
//! The `calendula caldav delete` command, running a WebDAV `DELETE`
//! against an item.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    caldav::client::{CaldavClient, item_refusal},
    shared::receipt::IdReceipt,
};

/// DELETE an item.
///
/// Deletes one item, gated on `--if-match`. A calendar collection is
/// deleted by `caldav rmcol`.
///
/// JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct CaldavDeleteCommand {
    /// Identifier of the calendar (its last URL segment).
    #[arg(value_name = "CALENDAR")]
    pub calendar_id: String,
    /// Item resource id (its href last path segment).
    #[arg(value_name = "ITEM-ID")]
    pub item_id: String,
    /// Gate the delete on the resource being unchanged (If-Match).
    #[arg(long, value_name = "ETAG")]
    pub if_match: Option<String>,
}

impl CaldavDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CaldavClient) -> Result<()> {
        client
            .delete_item(&self.calendar_id, &self.item_id, self.if_match.as_deref())
            .map_err(item_refusal)?;

        printer.out(IdReceipt {
            id: self.item_id,
            done: "Item deleted",
        })
    }
}
