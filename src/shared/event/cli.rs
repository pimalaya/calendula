//! # Event command
//!
//! The `event` command family, the VEVENT view over a calendar.
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
        event::{
            agenda::EventAgendaCommand, build::EventBuildCommand, create::EventCreateCommand,
            delete::EventDeleteCommand, find::EventFindCommand, list::EventListCommand,
            read::EventReadCommand, update::EventUpdateCommand,
        },
    },
};

/// Shared API to manage VEVENT items.
#[derive(Debug, Subcommand)]
pub enum EventCommand {
    Agenda(EventAgendaCommand),
    Build(Box<EventBuildCommand>),
    #[command(visible_alias = "ls")]
    List(EventListCommand),
    Read(EventReadCommand),
    Find(EventFindCommand),
    Create(Box<EventCreateCommand>),
    Update(Box<EventUpdateCommand>),
    Delete(EventDeleteCommand),
}

impl EventCommand {
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
            Self::Agenda(cmd) => {
                let client = CalendarClient::resolve(printer, config_paths, account_name, backend)?;
                cmd.execute(printer, client)
            }
            Self::List(cmd) => {
                let client = CalendarClient::resolve(printer, config_paths, account_name, backend)?;
                cmd.execute(printer, client)
            }
            Self::Read(cmd) => {
                let client = CalendarClient::resolve(printer, config_paths, account_name, backend)?;
                cmd.execute(printer, client)
            }
            Self::Find(cmd) => {
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
