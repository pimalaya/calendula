//! # Microsoft Graph commands
//!
//! Dispatches the Graph-specific command tree, one subtree per Graph
//! resource.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{
    calendars::cli::MsgraphCalendarsCommand, client::MsgraphClient,
    events::cli::MsgraphEventsCommand, profile::cli::MsgraphProfileCommand,
    request::MsgraphRequestCommand,
};

/// Microsoft Graph-specific API.
///
/// Nested by Graph resource (calendars, events, the signed-in user), each
/// command named after its Graph operation. Works with the raw Graph event
/// model: `create` / `update` take a Graph JSON body and `--json` prints
/// the raw Graph payload.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum MsgraphCommand {
    #[command(subcommand, alias = "calendar")]
    Calendars(MsgraphCalendarsCommand),
    #[command(subcommand, alias = "event")]
    Events(MsgraphEventsCommand),
    #[command(subcommand)]
    Profile(MsgraphProfileCommand),
    Request(MsgraphRequestCommand),
}

impl MsgraphCommand {
    pub fn execute(self, printer: &mut impl Printer, client: MsgraphClient) -> Result<()> {
        match self {
            Self::Calendars(cmd) => cmd.execute(printer, client),
            Self::Events(cmd) => cmd.execute(printer, client),
            Self::Profile(cmd) => cmd.execute(printer, client),
            Self::Request(cmd) => cmd.execute(printer, client),
        }
    }
}
