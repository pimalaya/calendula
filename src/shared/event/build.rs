//! # Event build
//!
//! The `event build` command: an event composed and printed, sent nowhere.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::shared::{build::IcalBuildArgs, ical::IcalFamily};

/// Build an event and print it, writing it nowhere.
///
/// The source and `-i` stack as they do on `event create`, and the event
/// comes back on stdout instead of going to a backend, which is how an
/// event is judged before it is sent. No account is resolved unless
/// `-i` needs the configured composer.
///
/// JSON output: `{"contents"}`, the raw iCalendar as text, or
/// `{"message"}` with `-o`.
#[derive(Debug, Parser)]
pub struct EventBuildCommand {
    #[command(flatten)]
    pub args: IcalBuildArgs,
}

impl EventBuildCommand {
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
    ) -> Result<()> {
        self.args
            .execute(printer, config_paths, account_name, IcalFamily::Event)
    }
}
