//! # Profile commands
//!
//! Dispatches the profile subtree.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{client::MsgraphClient, profile::get::MsgraphProfileGetCommand};

/// The signed-in user (`/me`).
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum MsgraphProfileCommand {
    Get(MsgraphProfileGetCommand),
}

impl MsgraphProfileCommand {
    pub fn execute(self, printer: &mut impl Printer, client: MsgraphClient) -> Result<()> {
        match self {
            Self::Get(cmd) => cmd.execute(printer, client),
        }
    }
}
