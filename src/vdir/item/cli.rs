//! # Vdir item command
//!
//! Declares the item command tree and dispatches each verb against the
//! account's vdir client.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::vdir::{
    client::VdirClient,
    item::{
        copy::VdirItemCopyCommand, create::VdirItemCreateCommand, delete::VdirItemDeleteCommand,
        get::VdirItemGetCommand, list::VdirItemListCommand, r#move::VdirItemMoveCommand,
        update::VdirItemUpdateCommand,
    },
};

/// Manage raw vdir items, the files inside a collection.
///
/// Unlike the unified `item` API, these operate on the filesystem items
/// byte for byte and of any kind, vCard or iCalendar, surfacing the
/// on-disk id and kind.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum VdirItemCommand {
    #[command(visible_alias = "ls")]
    List(VdirItemListCommand),
    Get(VdirItemGetCommand),
    #[command(visible_alias = "new")]
    Create(VdirItemCreateCommand),
    Update(VdirItemUpdateCommand),
    #[command(visible_alias = "rm")]
    Delete(VdirItemDeleteCommand),
    Copy(VdirItemCopyCommand),
    #[command(name = "move")]
    Move(VdirItemMoveCommand),
}

impl VdirItemCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.execute(printer, client),
            Self::Get(cmd) => cmd.execute(printer, client),
            Self::Create(cmd) => cmd.execute(printer, client),
            Self::Update(cmd) => cmd.execute(printer, client),
            Self::Delete(cmd) => cmd.execute(printer, client),
            Self::Copy(cmd) => cmd.execute(printer, client),
            Self::Move(cmd) => cmd.execute(printer, client),
        }
    }
}
