use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::shared::{
    client::CalendarClient,
    journals::{
        create::JournalCreateCommand, delete::JournalDeleteCommand, list::JournalListCommand,
        read::JournalReadCommand, update::JournalUpdateCommand,
    },
};

/// Shared API to manage VJOURNAL items: list, read, create, update,
/// delete.
#[derive(Debug, Subcommand)]
pub enum JournalCommand {
    #[command(visible_alias = "ls")]
    List(JournalListCommand),
    Read(JournalReadCommand),
    Create(JournalCreateCommand),
    Update(JournalUpdateCommand),
    Delete(JournalDeleteCommand),
}

impl JournalCommand {
    pub fn execute(self, printer: &mut impl Printer, client: CalendarClient) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.execute(printer, client),
            Self::Read(cmd) => cmd.execute(printer, client),
            Self::Create(cmd) => cmd.execute(printer, client),
            Self::Update(cmd) => cmd.execute(printer, client),
            Self::Delete(cmd) => cmd.execute(printer, client),
        }
    }
}
