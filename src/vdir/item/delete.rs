//! # Item delete command
//!
//! Removes one item file from a collection.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{shared::receipt::IdReceipt, vdir::client::VdirClient};

/// Delete the given item from the collection.
///
/// JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct VdirItemDeleteCommand {
    /// Collection the item lives in.
    #[arg(value_name = "COLLECTION")]
    pub collection: String,
    /// Item id (file stem, without extension).
    #[arg(value_name = "ID")]
    pub id: String,
}

impl VdirItemDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let path = client.collection_path(&self.collection)?;
        client.delete_item(path, &self.id)?;

        printer.out(IdReceipt {
            id: self.id,
            done: "Item deleted",
        })
    }
}
