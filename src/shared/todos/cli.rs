use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::shared::{
    client::CalendarClient,
    todos::{
        create::TodoCreateCommand, delete::TodoDeleteCommand, list::TodoListCommand,
        read::TodoReadCommand, update::TodoUpdateCommand,
    },
};

/// Shared API to manage VTODO items: list, read, create, update,
/// delete.
#[derive(Debug, Subcommand)]
pub enum TodoCommand {
    #[command(visible_alias = "ls")]
    List(TodoListCommand),
    Read(TodoReadCommand),
    Create(TodoCreateCommand),
    Update(TodoUpdateCommand),
    Delete(TodoDeleteCommand),
}

impl TodoCommand {
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
