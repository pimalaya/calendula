//! # Todo command
//!
//! The `todo` command family, the VTODO view over a calendar.
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
        todo::{
            build::TodoBuildCommand, create::TodoCreateCommand, delete::TodoDeleteCommand,
            list::TodoListCommand, read::TodoReadCommand, update::TodoUpdateCommand,
        },
    },
};

/// Shared API to manage VTODO items.
#[derive(Debug, Subcommand)]
pub enum TodoCommand {
    Build(TodoBuildCommand),
    #[command(visible_alias = "ls")]
    List(TodoListCommand),
    Read(TodoReadCommand),
    Create(TodoCreateCommand),
    Update(TodoUpdateCommand),
    Delete(TodoDeleteCommand),
}

impl TodoCommand {
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
