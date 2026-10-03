//! # Pimdir CLI
//!
//! The `pimdir` command family, reporting on the local store the
//! account reads from.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::pimdir::{
    backend::PimdirBackend,
    intent::{PimdirCancelCommand, PimdirReplyCommand},
    status::PimdirStatusCommand,
};

/// pimdir CLI.
///
/// Direct access to the local pimdir store behind the account: what it
/// holds and how much of it is downloaded, and the scheduling messages
/// (a reply, a cancellation) only its sync engine can send.
///
/// The store's own operator tooling, the `pimdir` binary io-pimdir
/// ships, covers the rest, including the queue and the retained items.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum PimdirCommand {
    Status(PimdirStatusCommand),
    Reply(PimdirReplyCommand),
    Cancel(PimdirCancelCommand),
}

impl PimdirCommand {
    pub fn execute(self, printer: &mut impl Printer, backend: PimdirBackend) -> Result<()> {
        match self {
            Self::Status(cmd) => cmd.execute(printer, backend),
            Self::Reply(cmd) => cmd.execute(printer, backend),
            Self::Cancel(cmd) => cmd.execute(printer, backend),
        }
    }
}
