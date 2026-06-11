use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::shared::{
    calendars::{
        create::CalendarCreateCommand, delete::CalendarDeleteCommand, list::CalendarListCommand,
        update::CalendarUpdateCommand,
    },
    client::CalendarClient,
};

/// Shared API to manage calendars for the active account.
#[derive(Debug, Subcommand)]
pub enum CalendarCommand {
    #[command(visible_alias = "ls")]
    List(CalendarListCommand),
    Create(CalendarCreateCommand),
    Update(CalendarUpdateCommand),
    Delete(CalendarDeleteCommand),
}

impl CalendarCommand {
    pub fn execute(self, printer: &mut impl Printer, client: CalendarClient) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.execute(printer, client),
            Self::Create(cmd) => cmd.execute(printer, client),
            Self::Update(cmd) => cmd.execute(printer, client),
            Self::Delete(cmd) => cmd.execute(printer, client),
        }
    }
}
