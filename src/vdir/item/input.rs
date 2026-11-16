//! # Item input
//!
//! The item source shared by the add and update commands, plus the kind
//! selector and the sniffing that infers a kind from the bytes.

use std::{
    fs,
    io::{Read, stdin},
    path::PathBuf,
};

use anyhow::{Context, Result, bail};
use clap::{Parser, ValueEnum};
use io_vdir::item::VdirItemKind;

/// Positional item source shared by `item add` and `item update`.
#[derive(Debug, Parser)]
pub struct ItemInputArg {
    /// A path to an item file, raw item contents, or `-` for stdin.
    #[arg(value_name = "INPUT")]
    pub input: String,
}

impl ItemInputArg {
    /// Resolves the source into raw bytes.
    ///
    /// `-` reads stdin, an existing path is read, and anything else is
    /// taken as literal item contents.
    pub fn read(self) -> Result<Vec<u8>> {
        if self.input == "-" {
            pimalaya_cli::session::check_stdin(crate::error::ErrorCode::StdinTaken)?;
            let mut buf = Vec::new();
            stdin()
                .read_to_end(&mut buf)
                .context("Read item from stdin error")?;
            return Ok(buf);
        }

        let path = PathBuf::from(&self.input);
        if path.is_file() {
            return fs::read(&path)
                .with_context(|| format!("Read item from `{}` error", path.display()));
        }

        if self.input.trim_start().starts_with("BEGIN:V") {
            return Ok(self.input.into_bytes());
        }

        bail!(
            "Source `{}` is neither a readable file nor item contents",
            self.input
        )
    }
}

/// Item kind selector for `item add`.
#[derive(Clone, Copy, Debug, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum ItemKindArg {
    /// An iCalendar item, stored with the `.ics` extension.
    Ical,
    /// A vCard item, stored with the `.vcf` extension.
    Vcard,
}

impl From<ItemKindArg> for VdirItemKind {
    fn from(arg: ItemKindArg) -> Self {
        match arg {
            ItemKindArg::Ical => Self::Ical,
            ItemKindArg::Vcard => Self::Vcard,
        }
    }
}

/// Wire name of an item kind, for display and JSON.
pub fn kind_str(kind: VdirItemKind) -> &'static str {
    match kind {
        VdirItemKind::Ical => "ical",
        VdirItemKind::Vcard => "vcard",
    }
}

/// Sniffs the item kind from its bytes.
///
/// A `BEGIN:VCARD` head is vCard, everything else iCalendar, the kind a
/// calendar holds.
pub fn sniff_kind(contents: &[u8]) -> VdirItemKind {
    let head = contents.get(..contents.len().min(64)).unwrap_or(contents);
    if String::from_utf8_lossy(head)
        .trim_start()
        .to_ascii_uppercase()
        .starts_with("BEGIN:VCARD")
    {
        VdirItemKind::Vcard
    } else {
        VdirItemKind::Ical
    }
}
