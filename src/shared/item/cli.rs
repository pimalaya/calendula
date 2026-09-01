//! # Item command
//!
//! The `item` command family, the raw and unfiltered view over a calendar:
//! it addresses any iCalendar object by id and leaves the bytes untouched.
//!
//! Every subcommand but `build` resolves the account it runs against;
//! `build` reaches no backend and needs none.

use std::path::PathBuf;

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::{
    backend::Backend,
    shared::{
        client::CalendarClient,
        item::{
            build::ItemBuildCommand, create::ItemCreateCommand, delete::ItemDeleteCommand,
            list::ItemListCommand, read::ItemReadCommand, update::ItemUpdateCommand,
        },
    },
};

/// Shared API to manage raw iCalendar items (VEVENT, VTODO, VJOURNAL).
#[derive(Debug, Subcommand)]
pub enum ItemCommand {
    Build(ItemBuildCommand),
    #[command(visible_alias = "ls")]
    List(ItemListCommand),
    Read(ItemReadCommand),
    Create(ItemCreateCommand),
    Update(ItemUpdateCommand),
    Delete(ItemDeleteCommand),
}

impl ItemCommand {
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
        backend: Backend,
    ) -> Result<()> {
        match self {
            // NOTE: `build` reaches no backend, so it resolves no account
            // either: printing an iCalendar on a machine holding no
            // configuration is what it is for.
            Self::Build(cmd) => cmd.execute(printer, config_paths, account_name),
            Self::List(cmd) => {
                let client = CalendarClient::resolve(printer, config_paths, account_name, backend)?;
                cmd.execute(printer, client)
            }
            Self::Read(cmd) => {
                let client = CalendarClient::resolve(printer, config_paths, account_name, backend)?;
                cmd.execute(printer, client)
            }
            Self::Create(cmd) => {
                let client = CalendarClient::resolve(printer, config_paths, account_name, backend)?;
                cmd.execute(printer, client)
            }
            Self::Update(cmd) => {
                let client = CalendarClient::resolve(printer, config_paths, account_name, backend)?;
                cmd.execute(printer, client)
            }
            Self::Delete(cmd) => {
                let client = CalendarClient::resolve(printer, config_paths, account_name, backend)?;
                cmd.execute(printer, client)
            }
        }
    }
}
