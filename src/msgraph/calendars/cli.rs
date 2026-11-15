//! # Calendar commands
//!
//! Dispatches the calendar subtree.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{
    calendars::{
        create::MsgraphCalendarCreateCommand, delete::MsgraphCalendarDeleteCommand,
        get::MsgraphCalendarGetCommand, list::MsgraphCalendarListCommand,
        rename::MsgraphCalendarRenameCommand,
    },
    client::MsgraphClient,
};

/// Manage Graph calendars.
///
/// Graph calendars do not nest, so no child listing stands beside them as
/// it does beside contact folders.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum MsgraphCalendarsCommand {
    List(MsgraphCalendarListCommand),
    Get(MsgraphCalendarGetCommand),
    #[command(visible_aliases = ["add", "new"])]
    Create(MsgraphCalendarCreateCommand),
    Rename(MsgraphCalendarRenameCommand),
    #[command(visible_aliases = ["del", "rm"])]
    Delete(MsgraphCalendarDeleteCommand),
}

impl MsgraphCalendarsCommand {
    pub fn execute(self, printer: &mut impl Printer, client: MsgraphClient) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.execute(printer, client),
            Self::Get(cmd) => cmd.execute(printer, client),
            Self::Create(cmd) => cmd.execute(printer, client),
            Self::Rename(cmd) => cmd.execute(printer, client),
            Self::Delete(cmd) => cmd.execute(printer, client),
        }
    }
}
