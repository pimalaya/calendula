//! # Item create command
//!
//! Stores a new item file in a collection, inferring its kind when the
//! command does not give one.

use core::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::vdir::{
    client::VdirClient,
    item::input::{ItemInputArg, ItemKindArg, sniff_kind},
};

/// Store a new item in the given collection.
///
/// The kind, which is the file extension, defaults to sniffing the input:
/// a `BEGIN:VCARD` head is vCard, everything else iCalendar. Override it
/// with `--kind`. JSON output: `{"id"}`, the file stem the item landed
/// under.
#[derive(Debug, Parser)]
pub struct VdirItemCreateCommand {
    /// Collection to store the item in.
    #[arg(value_name = "COLLECTION")]
    pub collection: String,
    /// Item kind; inferred from the input when omitted.
    #[arg(long, value_name = "KIND")]
    pub kind: Option<ItemKindArg>,
    #[command(flatten)]
    pub input: ItemInputArg,
}

impl VdirItemCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let path = client.collection_path(&self.collection)?;
        let contents = self.input.read()?;
        let kind = match self.kind {
            Some(kind) => kind.into(),
            None => sniff_kind(&contents),
        };

        let (id, _) = client.store_item(path, None, kind, contents)?;

        printer.out(VdirItemCreateOutput { id })
    }
}

/// The item the collection now holds.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VdirItemCreateOutput {
    /// Item id, the file stem it was stored under.
    pub id: String,
}

impl fmt::Display for VdirItemCreateOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Item `{}` successfully created", self.id)
    }
}
