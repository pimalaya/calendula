use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::shared::{
    client::CalendarClient,
    items::{
        create::ItemCreateCommand, delete::ItemDeleteCommand, list::ItemListCommand,
        read::ItemReadCommand, update::ItemUpdateCommand,
    },
};

/// Shared API to manage raw iCalendar items (VEVENT, VTODO, VJOURNAL).
#[derive(Debug, Subcommand)]
pub enum ItemCommand {
    #[command(visible_alias = "ls")]
    List(ItemListCommand),
    Read(ItemReadCommand),
    Create(ItemCreateCommand),
    Update(ItemUpdateCommand),
    Delete(ItemDeleteCommand),
}

impl ItemCommand {
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
