//! # Event command
//!
//! The `event` command family, the VEVENT view over a calendar.
//!
//! Every subcommand but `build` runs over the session's unified client;
//! `build` reaches no backend and needs none.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::{
    capability,
    session::Session,
    shared::event::{
        agenda::EventAgendaCommand, build::EventBuildCommand, create::EventCreateCommand,
        delete::EventDeleteCommand, find::EventFindCommand, list::EventListCommand,
        read::EventReadCommand, update::EventUpdateCommand,
    },
};

/// Manage events (VEVENT) using the unified API.
#[derive(Debug, Subcommand)]
pub enum EventCommand {
    #[command(after_long_help = capability::help("event", "agenda"))]
    Agenda(EventAgendaCommand),
    Build(Box<EventBuildCommand>),
    #[command(visible_alias = "ls", after_long_help = capability::help("event", "list"))]
    List(EventListCommand),
    #[command(after_long_help = capability::help("event", "read"))]
    Read(EventReadCommand),
    #[command(after_long_help = capability::help("event", "find"))]
    Find(EventFindCommand),
    #[command(after_long_help = capability::help("event", "create"))]
    Create(Box<EventCreateCommand>),
    #[command(after_long_help = capability::help("event", "update"))]
    Update(Box<EventUpdateCommand>),
    #[command(after_long_help = capability::help("event", "delete"))]
    Delete(EventDeleteCommand),
}

impl EventCommand {
    /// Runs the subcommand, over the session's unified client but for
    /// `build`, which reaches no backend.
    pub fn execute(self, printer: &mut impl Printer, session: &mut Session) -> Result<()> {
        let account = session.account;

        match self {
            // NOTE: `build` reaches no backend, so it resolves no account
            // either: printing an iCalendar on a machine holding no
            // configuration is what it is for.
            Self::Build(cmd) => cmd.execute(printer, account.config_paths, account.name),
            Self::Agenda(cmd) => session.run_unified(printer, |p, client| cmd.execute(p, client)),
            Self::List(cmd) => session.run_unified(printer, |p, client| cmd.execute(p, client)),
            Self::Read(cmd) => session.run_unified(printer, |p, client| cmd.execute(p, client)),
            Self::Find(cmd) => session.run_unified(printer, |p, client| cmd.execute(p, client)),
            Self::Create(cmd) => session.run_unified(printer, |p, client| cmd.execute(p, client)),
            Self::Update(cmd) => session.run_unified(printer, |p, client| cmd.execute(p, client)),
            Self::Delete(cmd) => session.run_unified(printer, |p, client| cmd.execute(p, client)),
        }
    }
}
