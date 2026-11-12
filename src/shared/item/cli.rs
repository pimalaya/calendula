//! # Item command
//!
//! The `item` command family, the raw and unfiltered view over a calendar:
//! it addresses any iCalendar object by id and leaves the bytes untouched.
//!
//! Every subcommand but `build` runs over the session's unified client;
//! `build` reaches no backend and needs none.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::{
    capability,
    session::Session,
    shared::item::{
        build::ItemBuildCommand, create::ItemCreateCommand, delete::ItemDeleteCommand,
        list::ItemListCommand, read::ItemReadCommand, update::ItemUpdateCommand,
    },
};

/// Manage raw iCalendar items (any component) using the unified API.
#[derive(Debug, Subcommand)]
pub enum ItemCommand {
    Build(ItemBuildCommand),
    #[command(visible_alias = "ls", after_long_help = capability::help("item", "list"))]
    List(ItemListCommand),
    #[command(after_long_help = capability::help("item", "read"))]
    Read(ItemReadCommand),
    #[command(after_long_help = capability::help("item", "create"))]
    Create(ItemCreateCommand),
    #[command(after_long_help = capability::help("item", "update"))]
    Update(ItemUpdateCommand),
    #[command(after_long_help = capability::help("item", "delete"))]
    Delete(ItemDeleteCommand),
}

impl ItemCommand {
    /// Runs the subcommand, over the session's unified client but for
    /// `build`, which reaches no backend.
    pub fn execute(self, printer: &mut impl Printer, session: &mut Session) -> Result<()> {
        let account = session.account;

        match self {
            // NOTE: `build` reaches no backend, so it resolves no account
            // either: printing an iCalendar on a machine holding no
            // configuration is what it is for.
            Self::Build(cmd) => cmd.execute(printer, account.config_paths, account.name),
            Self::List(cmd) => session.run_unified(printer, |p, client| cmd.execute(p, client)),
            Self::Read(cmd) => session.run_unified(printer, |p, client| cmd.execute(p, client)),
            Self::Create(cmd) => session.run_unified(printer, |p, client| cmd.execute(p, client)),
            Self::Update(cmd) => session.run_unified(printer, |p, client| cmd.execute(p, client)),
            Self::Delete(cmd) => session.run_unified(printer, |p, client| cmd.execute(p, client)),
        }
    }
}
