//! # Event commands
//!
//! Dispatches the event subtree.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{
    client::MsgraphClient,
    events::{
        create::MsgraphEventCreateCommand, delete::MsgraphEventDeleteCommand,
        delta::MsgraphEventDeltaCommand, get::MsgraphEventGetCommand,
        list::MsgraphEventListCommand, update::MsgraphEventUpdateCommand,
    },
};

/// Manage Graph events.
///
/// `create` / `update` take a raw Graph event JSON body (file, inline, or
/// `-` for stdin); `--json` prints the raw Graph event.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum MsgraphEventsCommand {
    List(MsgraphEventListCommand),
    Get(MsgraphEventGetCommand),
    #[command(visible_aliases = ["add", "new"])]
    Create(MsgraphEventCreateCommand),
    Update(MsgraphEventUpdateCommand),
    #[command(visible_aliases = ["del", "rm"])]
    Delete(MsgraphEventDeleteCommand),
    Delta(MsgraphEventDeltaCommand),
}

impl MsgraphEventsCommand {
    pub fn execute(self, printer: &mut impl Printer, client: MsgraphClient) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.execute(printer, client),
            Self::Get(cmd) => cmd.execute(printer, client),
            Self::Create(cmd) => cmd.execute(printer, client),
            Self::Update(cmd) => cmd.execute(printer, client),
            Self::Delete(cmd) => cmd.execute(printer, client),
            Self::Delta(cmd) => cmd.execute(printer, client),
        }
    }
}
