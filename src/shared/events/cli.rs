use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::shared::{
    client::CalendarClient,
    events::{
        agenda::EventAgendaCommand, create::EventCreateCommand, delete::EventDeleteCommand,
        list::EventListCommand, read::EventReadCommand, update::EventUpdateCommand,
    },
};

/// Shared API to manage VEVENT items: agenda, list, read, create,
/// update, delete.
#[derive(Debug, Subcommand)]
pub enum EventCommand {
    Agenda(EventAgendaCommand),
    #[command(visible_alias = "ls")]
    List(EventListCommand),
    Read(EventReadCommand),
    Create(EventCreateCommand),
    Update(EventUpdateCommand),
    Delete(EventDeleteCommand),
}

impl EventCommand {
    pub fn execute(self, printer: &mut impl Printer, client: CalendarClient) -> Result<()> {
        match self {
            Self::Agenda(cmd) => cmd.execute(printer, client),
            Self::List(cmd) => cmd.execute(printer, client),
            Self::Read(cmd) => cmd.execute(printer, client),
            Self::Create(cmd) => cmd.execute(printer, client),
            Self::Update(cmd) => cmd.execute(printer, client),
            Self::Delete(cmd) => cmd.execute(printer, client),
        }
    }
}
