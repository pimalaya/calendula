use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::{CommandFactory, Parser, Subcommand};
use pimalaya_cli::{
    clap::{
        args::{AccountFlag, JsonFlag, LogFlags},
        commands::{CompletionCommand, ManualCommand},
        parsers::path_parser,
    },
    long_version,
    printer::Printer,
};
use pimalaya_config::toml::TomlConfig;

#[cfg(feature = "caldav")]
use crate::caldav::{cli::CaldavCommand, client::build_caldav_client};
#[cfg(feature = "vdir")]
use crate::vdir::{cli::VdirCommand, client::build_vdir_client};
use crate::{
    account::cli::AccountCommand,
    backend::Backend,
    config::Config,
    shared::{
        calendars::cli::CalendarCommand, client::CalendarClient, events::cli::EventCommand,
        items::cli::ItemCommand,
    },
    wizard,
};

#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(author, version, about)]
#[command(long_version = long_version!())]
#[command(propagate_version = true, infer_subcommands = true)]
pub struct CalendulaCli {
    #[command(subcommand)]
    pub command: CalendulaCommand,

    /// Override the default configuration file path.
    ///
    /// The given paths are shell-expanded then canonicalized (if
    /// applicable). If the first path does not point to a valid file,
    /// the wizard will propose to assist you in the creation of the
    /// configuration file. Other paths are merged with the first one,
    /// which allows you to separate your public config from your
    /// private(s) one(s). Multiple paths can also be provided by
    /// delimiting them with `:` (like `$PATH` in a POSIX shell).
    #[arg(short, long = "config", global = true, env = "CALENDULA_CONFIG")]
    #[arg(value_name = "PATH", value_parser = path_parser, value_delimiter = ':')]
    pub config_paths: Vec<PathBuf>,
    #[command(flatten)]
    pub account: AccountFlag,
    /// Force a specific backend for cross-protocol commands.
    ///
    /// Only consumed by the shared commands (`calendar`, `event`,
    /// `item`); the protocol-specific subcommands (`vdir`, `caldav`)
    /// ignore it and always use their own backend.
    ///
    /// Possible values: `auto` (default), `vdir`, `caldav`. With
    /// `auto`, the shared command picks the first configured backend
    /// it supports (vdir wins over caldav); with an explicit value, it
    /// uses only that backend (and bails if the account has no
    /// matching config block).
    #[arg(short, long, global = true, default_value_t)]
    pub backend: Backend,
    #[command(flatten)]
    pub json: JsonFlag,
    #[command(flatten)]
    pub log: LogFlags,
}

#[derive(Debug, Subcommand)]
pub enum CalendulaCommand {
    // --- Shared API
    //
    #[command(subcommand, visible_alias = "cal", alias = "calendars")]
    Calendar(CalendarCommand),
    #[command(subcommand, alias = "events")]
    Event(EventCommand),
    #[command(subcommand, alias = "items")]
    Item(ItemCommand),

    // --- Protocol-specific APIs
    //
    #[cfg(feature = "caldav")]
    #[command(subcommand)]
    Caldav(CaldavCommand),
    #[cfg(feature = "vdir")]
    #[command(subcommand)]
    Vdir(VdirCommand),

    // --- Meta
    //
    #[command(subcommand)]
    Account(AccountCommand),
    Completions(CompletionCommand),
    Manuals(ManualCommand),
}

/// Loads `Config` from the merged `config_paths` or, when no file
/// exists, runs the wizard to bootstrap one at the target path. Used
/// by every `build_*_client` helper to get a populated `Config` before
/// the per-backend client opens its connection.
pub fn load_or_wizard(config_paths: &[PathBuf]) -> Result<Config> {
    match Config::from_paths_or_default(config_paths)? {
        Some(config) => Ok(config),
        None => wizard::discover::run_or_exit(&Config::target_path(config_paths)?),
    }
}

impl CalendulaCommand {
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
        backend: Backend,
    ) -> Result<()> {
        let configs = || {
            let mut config = load_or_wizard(config_paths)?;

            let Some((_, account_config)) = config.take_account(account_name)? else {
                bail!("Cannot find default account; use --account or set account.default = true")
            };

            Ok((config, account_config))
        };

        match self {
            // --- Shared API
            //
            Self::Calendar(cmd) => {
                let (config, account_config) = configs()?;
                let client = CalendarClient::new(config, account_config, backend)?;
                cmd.execute(printer, client)
            }
            Self::Event(cmd) => {
                let (config, account_config) = configs()?;
                let client = CalendarClient::new(config, account_config, backend)?;
                cmd.execute(printer, client)
            }
            Self::Item(cmd) => {
                let (config, account_config) = configs()?;
                let client = CalendarClient::new(config, account_config, backend)?;
                cmd.execute(printer, client)
            }

            // --- Protocol-specific APIs
            //
            #[cfg(feature = "caldav")]
            Self::Caldav(cmd) => {
                let client = build_caldav_client(config_paths, account_name)?;
                cmd.execute(printer, client)
            }
            #[cfg(feature = "vdir")]
            Self::Vdir(cmd) => {
                let client = build_vdir_client(config_paths, account_name)?;
                cmd.execute(printer, client)
            }

            // --- Meta
            //
            Self::Account(cmd) => cmd.execute(printer, config_paths, account_name, backend),
            Self::Completions(cmd) => cmd.execute(printer, CalendulaCli::command()),
            Self::Manuals(cmd) => cmd.execute(printer, CalendulaCli::command()),
        }
    }
}
