//! # Vdir collection rename
//!
//! The `calendula vdir rename` command, renaming one collection
//! directory in place.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    shared::receipt::IdReceipt,
    vdir::client::{VdirClient, check_rename_target_absent},
};

/// Rename a vdir collection directory.
///
/// The id is the directory name, so this renames the directory itself
/// and leaves the item files inside it untouched. A new id the root holds
/// already is refused, coded `calendar-duplicate`.
///
/// JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct VdirCollectionRenameCommand {
    /// Current collection identifier (directory name).
    #[arg(value_name = "OLD-ID")]
    pub id: String,

    /// New collection identifier (directory name).
    #[arg(value_name = "NEW-ID")]
    pub new_id: String,
}

impl VdirCollectionRenameCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let path = client.collection_path(&self.id)?;
        check_rename_target_absent(client.root(), &self.id, &self.new_id)?;
        let new_id = self.new_id.clone();
        client.rename_collection(path, self.new_id)?;

        printer.out(IdReceipt {
            id: new_id,
            done: "Collection renamed",
        })
    }
}
