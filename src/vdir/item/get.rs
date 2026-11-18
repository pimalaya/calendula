//! # Item get command
//!
//! Reads one item and prints its bytes as they are stored.

use core::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    shared::item::{ItemContents, read::printable},
    vdir::{client::VdirClient, item::input::kind_str},
};

/// Read the raw bytes of the given item.
///
/// JSON output: `{"id", "kind", "contents", "encoding"}`, with the raw
/// item in `contents`: its text (`encoding` `utf-8`), or the base64 of
/// bytes that are not UTF-8 (`encoding` `base64`).
#[derive(Debug, Parser)]
pub struct VdirItemGetCommand {
    /// Collection the item lives in.
    #[arg(value_name = "COLLECTION")]
    pub collection: String,
    /// Item id (file stem, without extension).
    #[arg(value_name = "ID")]
    pub id: String,
}

impl VdirItemGetCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let path = client.collection_path(&self.collection)?;
        let item = client.get_item(path, &self.id)?;

        let out = VdirItemGetOutput {
            id: self.id,
            kind: kind_str(item.kind),
            contents: ItemContents::of(&item.contents),
            bytes: item.contents,
        };

        printer.out(out)
    }
}

/// One fetched item: its id, its kind and its raw contents.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VdirItemGetOutput {
    /// Item id, its file stem.
    pub id: String,
    /// The item kind its extension names.
    pub kind: &'static str,
    /// The item's contents, as text or as base64.
    #[serde(flatten)]
    pub contents: ItemContents,
    /// The bytes as stored, for the text view.
    #[serde(skip)]
    pub bytes: Vec<u8>,
}

impl fmt::Display for VdirItemGetOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", printable(&self.bytes).trim_end())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use pimalaya_cli::printer::CapturePrinter;
    use serde_json::json;
    use tempfile::TempDir;

    use super::VdirItemGetCommand;
    use crate::{config::VdirConfig, vdir::client::VdirClient};

    /// An item is printed as stored, its bytes as text when they are UTF-8
    /// and as base64 otherwise, never decoded lossily.
    #[test]
    fn an_item_prints_its_bytes_as_text_or_base64() {
        let root = TempDir::new().unwrap();
        fs::create_dir(root.path().join("cal")).unwrap();
        fs::write(root.path().join("cal/a.ics"), b"BEGIN:VCALENDAR\r\n").unwrap();
        fs::write(root.path().join("cal/b.ics"), b"SUMMARY:Caf\xe9\r\n").unwrap();

        let get = |id: &str| {
            let config = VdirConfig {
                root: root.path().to_owned(),
            };
            let cmd = VdirItemGetCommand {
                collection: String::from("cal"),
                id: id.to_owned(),
            };
            let mut printer = CapturePrinter::default();
            cmd.execute(&mut printer, VdirClient::new(config)).unwrap();
            printer.into_single().unwrap()
        };

        assert_eq!(
            get("a"),
            json!({"id": "a", "kind": "ical", "contents": "BEGIN:VCALENDAR\r\n", "encoding": "utf-8"})
        );
        assert_eq!(
            get("b"),
            json!({"id": "b", "kind": "ical", "contents": "U1VNTUFSWTpDYWbpDQo=", "encoding": "base64"})
        );
    }
}
