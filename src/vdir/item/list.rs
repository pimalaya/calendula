//! # Item list command
//!
//! Lists the raw items of a collection, of any kind, as text or JSON.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use io_vdir::item::VdirItem;
use pimalaya_cli::{columns::Columns, printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::vdir::{client::VdirClient, item::input::kind_str};

/// List every item in the given collection, of any kind.
///
/// Unlike `event list`, this includes every iCalendar component and vCard
/// items, and surfaces each item's kind. JSON output: `{"items": [{"id",
/// "kind", "size", "path"}]}`.
#[derive(Debug, Parser)]
pub struct VdirItemListCommand {
    /// Collection the items live in (final path segment under the root).
    #[arg(value_name = "COLLECTION")]
    pub collection: String,
}

impl VdirItemListCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let path = client.collection_path(&self.collection)?;
        let items = client.list_items(path)?;

        let output = VdirItemListOutput {
            rows: items.into_iter().filter_map(ItemRow::from_item).collect(),
        };

        printer.out(output)
    }
}

/// The item listing, as the text and the JSON both render it.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VdirItemListOutput {
    /// The items the collection holds.
    #[serde(rename = "items")]
    pub rows: Vec<ItemRow>,
}

/// One listed item: its id, kind, size and path on disk.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemRow {
    /// Item id, its file stem.
    pub id: String,
    /// The item kind its extension names.
    pub kind: &'static str,
    /// Size of its contents, in bytes.
    pub size: usize,
    /// The item's path on disk.
    pub path: String,
}

impl ItemRow {
    fn from_item(item: VdirItem) -> Option<Self> {
        let id = item.id()?.to_string();
        Some(Self {
            id,
            kind: kind_str(item.kind),
            size: item.contents.len(),
            path: item.path.as_str().to_string(),
        })
    }
}

impl fmt::Display for VdirItemListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.rows.is_empty() {
            return Ok(());
        }

        let mut columns = Columns::new(["ID", "KIND", "SIZE", "PATH"]);

        for item in &self.rows {
            columns.push([
                sanitize::text(&item.id).into_owned(),
                item.kind.to_string(),
                item.size.to_string(),
                sanitize::text(&item.path).into_owned(),
            ]);
        }

        write!(f, "{columns}")
    }
}
