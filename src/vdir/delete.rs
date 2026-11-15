//! # Vdir collection delete
//!
//! The `calendula vdir delete` command, removing one collection
//! directory and everything inside it.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{shared::receipt::IdReceipt, vdir::client::VdirClient};

/// Delete a vdir collection.
///
/// Removes the collection directory and every item file in it, without
/// asking and without a trash to recover them from.
///
/// JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct VdirCollectionDeleteCommand {
    /// Collection identifier (directory name).
    #[arg(value_name = "ID")]
    pub id: String,
}

impl VdirCollectionDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let path = client.collection_path(&self.id)?;
        client.delete_collection(path)?;

        printer.out(IdReceipt {
            id: self.id,
            done: "Collection deleted",
        })
    }
}
