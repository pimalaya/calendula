//! # Item update command
//!
//! Overwrites the bytes of an existing item, keeping the kind it is
//! stored under.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    shared::receipt::IdReceipt,
    vdir::{client::VdirClient, item::input::ItemInputArg},
};

/// Overwrite the bytes of an existing item.
///
/// The kind, which is the file extension, is read from the current item
/// and preserved. JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct VdirItemUpdateCommand {
    /// Collection the item lives in.
    #[arg(value_name = "COLLECTION")]
    pub collection: String,
    /// Item id (file stem, without extension).
    #[arg(value_name = "ID")]
    pub id: String,
    #[command(flatten)]
    pub input: ItemInputArg,
}

impl VdirItemUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let path = client.collection_path(&self.collection)?;
        let kind = client.get_item(path.clone(), &self.id)?.kind;
        let contents = self.input.read()?;

        client.store_item(path, Some(self.id.clone()), kind, contents)?;

        printer.out(IdReceipt {
            id: self.id,
            done: "Item updated",
        })
    }
}
