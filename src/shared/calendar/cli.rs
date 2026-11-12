//! # Calendar command
//!
//! The `calendar` command family, the portable view over the collections
//! an account holds.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::{
    capability,
    shared::{
        calendar::{
            create::CalendarCreateCommand, delete::CalendarDeleteCommand,
            list::CalendarListCommand, update::CalendarUpdateCommand,
        },
        client::CalendarClient,
    },
};

/// Manage calendars using the unified API.
#[derive(Debug, Subcommand)]
pub enum CalendarCommand {
    #[command(visible_alias = "ls", after_long_help = capability::help("calendar", "list"))]
    List(CalendarListCommand),
    #[command(after_long_help = capability::help("calendar", "create"))]
    Create(CalendarCreateCommand),
    #[command(after_long_help = capability::help("calendar", "update"))]
    Update(CalendarUpdateCommand),
    #[command(after_long_help = capability::help("calendar", "delete"))]
    Delete(CalendarDeleteCommand),
}

impl CalendarCommand {
    pub fn execute(self, printer: &mut impl Printer, client: &mut CalendarClient) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.execute(printer, client),
            Self::Create(cmd) => cmd.execute(printer, client),
            Self::Update(cmd) => cmd.execute(printer, client),
            Self::Delete(cmd) => cmd.execute(printer, client),
        }
    }
}
