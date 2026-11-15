//! # DELETE command
//!
//! The `calendula caldav delete` command, running a WebDAV `DELETE`
//! against an item or a whole calendar collection.

use anyhow::{Result, bail};
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    caldav::client::{CaldavClient, item_refusal},
    shared::receipt::IdReceipt,
};

/// DELETE an item, or a whole calendar collection.
///
/// With an item id, deletes that item, gated on `--if-match`; without
/// one, deletes the calendar collection and every resource filed in it.
///
/// JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct CaldavDeleteCommand {
    /// Identifier of the calendar (its last URL segment).
    #[arg(value_name = "CALENDAR")]
    pub calendar_id: String,
    /// Item resource id; omit to delete the whole collection.
    #[arg(value_name = "ITEM-ID")]
    pub item_id: Option<String>,
    /// Gate an item delete on the resource being unchanged (If-Match).
    #[arg(long, value_name = "ETAG")]
    pub if_match: Option<String>,
}

impl CaldavDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CaldavClient) -> Result<()> {
        let receipt = match self.item_id {
            Some(item_id) => {
                client
                    .delete_item(&self.calendar_id, &item_id, self.if_match.as_deref())
                    .map_err(item_refusal)?;
                IdReceipt {
                    id: item_id,
                    done: "Item deleted",
                }
            }
            None => {
                if self.if_match.is_some() {
                    bail!("--if-match applies to an item delete, not to a collection");
                }
                client.delete_calendar(&self.calendar_id)?;
                IdReceipt {
                    id: self.calendar_id,
                    done: "Calendar deleted",
                }
            }
        };

        printer.out(receipt)
    }
}
