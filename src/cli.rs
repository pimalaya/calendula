//! The command tree and its single dispatch point.

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
#[cfg(feature = "pimdir")]
use crate::pimdir::{backend::PimdirBackend, cli::PimdirCommand};
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
    /// The command to run. Without one, the configuration wizard runs.
    #[command(subcommand)]
    pub command: Option<CalendulaCommand>,

    /// Override the default configuration file path.
    ///
    /// The given paths are shell-expanded then canonicalized when
    /// applicable. Several paths can be passed at once, separated by
    /// `:` like `$PATH` in a POSIX shell: the first one is the base and
    /// the rest are deep-merged on top, which is how a public
    /// configuration and a private one stay separate files.
    #[arg(short, long = "config", global = true, env = "CALENDULA_CONFIG")]
    #[arg(value_name = "PATH", value_parser = path_parser, value_delimiter = ':')]
    pub config_paths: Vec<PathBuf>,
    #[command(flatten)]
    pub account: AccountFlag,
    /// Force a specific backend for the cross-protocol commands.
    ///
    /// Only the shared commands (`calendar`, `event`, `item`) read it;
    /// the protocol-specific subcommands each already name their
    /// backend and ignore it.
    ///
    /// With `auto`, the first configured backend wins, in calendula's
    /// priority order (vdir, then pimdir, then caldav). With an
    /// explicit value, only that backend is used, and the command bails
    /// when the account carries no matching block.
    #[arg(short, long, global = true, default_value_t)]
    pub backend: Backend,
    #[command(flatten)]
    pub json: JsonFlag,
    #[command(flatten)]
    pub log: LogFlags,
}

/// Every command calendula serves, in three groups: the shared
/// cross-protocol API, the protocol-specific escape hatches, and the
/// meta commands.
#[derive(Debug, Subcommand)]
pub enum CalendulaCommand {
    #[command(subcommand, visible_alias = "cal", alias = "calendars")]
    Calendar(CalendarCommand),
    #[command(subcommand, alias = "events")]
    Event(EventCommand),
    #[command(subcommand, alias = "items")]
    Item(ItemCommand),

    #[cfg(feature = "caldav")]
    #[command(subcommand)]
    Caldav(CaldavCommand),
    #[cfg(feature = "pimdir")]
    #[command(subcommand)]
    Pimdir(PimdirCommand),
    #[cfg(feature = "vdir")]
    #[command(subcommand)]
    Vdir(VdirCommand),

    #[command(subcommand)]
    Account(AccountCommand),
    Completions(CompletionCommand),
    Manuals(ManualCommand),
}

/// Loads the configuration from the merged `config_paths`, or explains
/// how to get one.
///
/// Unlike the previous behaviour, a missing configuration is an error
/// rather than an implicit wizard run: the wizard prints a document
/// instead of writing one, so it cannot serve a command that is already
/// underway. Bare `calendula` runs it.
pub fn load_config(config_paths: &[PathBuf]) -> Result<Config> {
    match Config::from_paths_or_default(config_paths)? {
        Some(config) => Ok(config),
        None => bail!(
            "No configuration found. Run `calendula` with no command to generate one \
             with the wizard, or write one by hand."
        ),
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
            let mut config = load_config(config_paths)?;

            let Some((_, account_config)) = config.take_account(account_name)? else {
                bail!("Cannot find default account; use --account or set `default = true`")
            };

            Ok((config, account_config))
        };

        match self {
            Self::Calendar(cmd) => {
                let (config, account_config) = configs()?;
                cmd.execute(
                    printer,
                    CalendarClient::new(config, account_config, backend)?,
                )
            }
            Self::Event(cmd) => {
                let (config, account_config) = configs()?;
                cmd.execute(
                    printer,
                    CalendarClient::new(config, account_config, backend)?,
                )
            }
            Self::Item(cmd) => {
                let (config, account_config) = configs()?;
                cmd.execute(
                    printer,
                    CalendarClient::new(config, account_config, backend)?,
                )
            }

            #[cfg(feature = "caldav")]
            Self::Caldav(cmd) => {
                cmd.execute(printer, build_caldav_client(config_paths, account_name)?)
            }
            #[cfg(feature = "pimdir")]
            Self::Pimdir(cmd) => {
                cmd.execute(printer, PimdirBackend::build(config_paths, account_name)?)
            }
            #[cfg(feature = "vdir")]
            Self::Vdir(cmd) => cmd.execute(printer, build_vdir_client(config_paths, account_name)?),

            Self::Account(cmd) => cmd.execute(printer, config_paths, account_name, backend),
            Self::Completions(cmd) => cmd.execute(printer, CalendulaCli::command()),
            Self::Manuals(cmd) => cmd.execute(printer, CalendulaCli::command()),
        }
    }
}

/// Runs the parsed CLI: a subcommand, or the wizard when none was
/// given.
pub fn execute(cli: CalendulaCli, printer: &mut impl Printer) -> Result<()> {
    match cli.command {
        Some(command) => command.execute(
            printer,
            &cli.config_paths,
            cli.account.name.as_deref(),
            cli.backend,
        ),
        None => wizard::discover::run(printer),
    }
}
