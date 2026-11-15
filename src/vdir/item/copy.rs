//! # Item copy command
//!
//! Copies an item file from one collection into another.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{shared::receipt::IdReceipt, vdir::client::VdirClient};

/// Copy an item from one collection into another.
///
/// JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct VdirItemCopyCommand {
    /// Source collection the item lives in.
    #[arg(value_name = "SOURCE")]
    pub source: String,
    /// Target collection to copy the item into.
    #[arg(value_name = "TARGET")]
    pub target: String,
    /// Item id (file stem, without extension).
    #[arg(value_name = "ID")]
    pub id: String,
}

impl VdirItemCopyCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let source = client.collection_path(&self.source)?;
        let target = client.collection_path(&self.target)?;
        client.copy_item(source, target, &self.id)?;

        printer.out(IdReceipt {
            id: self.id,
            done: "Item copied",
        })
    }
}
