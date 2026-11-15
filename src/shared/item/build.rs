//! # Item build
//!
//! The `item build` command: an item composed and printed, sent nowhere.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::shared::{build::IcalBuildArgs, ical::IcalFamily};

/// Build an item and print it, writing it nowhere.
///
/// The source and `-i` stack as they do on `item add`, and the item
/// comes back on stdout instead of going to a backend, which is how an
/// item is judged before it is sent. No account is resolved unless
/// `-i` needs the configured composer.
///
/// JSON output: `{"contents", "encoding"}`, the raw iCalendar as text
/// (`encoding` `utf-8`) or the base64 of bytes that are not UTF-8
/// (`encoding` `base64`), which the text prints as they are; or `{"path",
/// "size"}` with `-o`.
#[derive(Debug, Parser)]
pub struct ItemBuildCommand {
    #[command(flatten)]
    pub args: IcalBuildArgs,
}

impl ItemBuildCommand {
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
    ) -> Result<()> {
        self.args
            .execute(printer, config_paths, account_name, IcalFamily::Item)
    }
}
