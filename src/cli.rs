//! # CLI
//!
//! The command tree calendula serves and its single dispatch point,
//! plus the configuration loading every command starts from.

use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::{CommandFactory, Parser, Subcommand};
use pimalaya_cli::{
    clap::{
        args::{AccountFlag, JsonFlag, LogFlags},
        commands::{CompletionCommand, JsonSchemaCommand, ManualCommand},
        parsers::path_parser,
    },
    long_version,
    printer::Printer,
};
use pimalaya_config::toml::TomlConfig;

#[cfg(feature = "caldav")]
use crate::caldav::{cli::CaldavCommand, client::build_caldav_client};
#[cfg(feature = "gcal")]
use crate::gcal::{cli::GcalCommand, client::build_gcal_client};
#[cfg(feature = "pimdir")]
use crate::pimdir::{backend::PimdirBackend, cli::PimdirCommand};
#[cfg(feature = "vdir")]
use crate::vdir::{cli::VdirCommand, client::build_vdir_client};
use crate::{
    account::cli::AccountCommand,
    backend::Backend,
    config::{AccountConfig, Config, NO_CONFIG_HINT},
    json_schema,
    shared::{
        calendar::cli::CalendarCommand, client::CalendarClient, event::cli::EventCommand,
        item::cli::ItemCommand,
    },
};

#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(author, version, about)]
#[command(long_version = long_version!())]
#[command(propagate_version = true, infer_subcommands = true)]
pub struct CalendulaCli {
    /// The command to run, a bare `calendula` showing this help.
    #[command(subcommand)]
    pub command: Option<CalendulaCommand>,

    /// Override the default configuration file path.
    ///
    /// Paths are shell-expanded then canonicalized. Several can be
    /// passed at once, `:`-delimited like `$PATH`, and are deep-merged
    /// left to right: that is how a public configuration and a private
    /// one stay separate files.
    #[arg(short, long = "config", global = true, env = "CALENDULA_CONFIG")]
    #[arg(value_name = "PATH", value_parser = path_parser, value_delimiter = ':')]
    pub config_paths: Vec<PathBuf>,
    #[command(flatten)]
    pub account: AccountFlag,
    /// Force a specific backend for the cross-protocol commands.
    ///
    /// Only the unified commands (`calendar`, `event`, `item`) read it:
    /// the protocol-specific subcommands each already name their
    /// backend.
    ///
    /// With `auto`, the first configured backend wins, in calendula's
    /// priority order (vdir, pimdir, caldav, gcal). Any other value
    /// pins that one, and the command bails when the account carries
    /// no matching block.
    #[arg(short, long, global = true, default_value_t)]
    pub backend: Backend,
    #[command(flatten)]
    pub json: JsonFlag,
    #[command(flatten)]
    pub log: LogFlags,
}

/// Every command calendula serves: the unified API, the
/// protocol-specific escape hatches, then the meta commands.
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
    #[cfg(feature = "gcal")]
    #[command(subcommand)]
    Gcal(GcalCommand),
    #[cfg(feature = "pimdir")]
    #[command(subcommand)]
    Pimdir(PimdirCommand),
    #[cfg(feature = "vdir")]
    #[command(subcommand)]
    Vdir(VdirCommand),
    #[command(subcommand)]
    Account(AccountCommand),
    #[command(alias = "completions")]
    Completion(CompletionCommand),
    #[command(alias = "manuals")]
    Manual(ManualCommand),
    #[command(alias = "json-schemas")]
    JsonSchema(JsonSchemaCommand),
}

/// The global config and the active account's, for a unified command.
///
/// Public so a family resolves its own account per subcommand: `build`
/// resolves none at all.
pub fn resolve_account(
    config_paths: &[PathBuf],
    account_name: Option<&str>,
) -> Result<(Config, AccountConfig)> {
    let mut config = load_config(config_paths)?;

    let Some((_, account_config)) = config.take_account(account_name)? else {
        bail!("Cannot find default account; use --account or set `default = true`")
    };

    Ok((config, account_config))
}

/// Loads the configuration from the merged `config_paths`.
///
/// A missing configuration fails naming the path looked at, which is
/// where `-c` pointed, so a mistyped path shows up as itself.
pub fn load_config(config_paths: &[PathBuf]) -> Result<Config> {
    match Config::from_paths_or_default(config_paths)? {
        Some(config) => Ok(config),
        None => bail!(
            "No configuration found at {}, {NO_CONFIG_HINT}",
            Config::target_path(config_paths)?.display()
        ),
    }
}

impl CalendulaCommand {
    /// Runs the subcommand against the account `-a` names, or the
    /// default one.
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
        backend: Backend,
    ) -> Result<()> {
        match self {
            Self::Calendar(cmd) => {
                let client = CalendarClient::resolve(config_paths, account_name, backend)?;
                cmd.execute(printer, client)
            }

            // NOTE: `event` and `item` resolve per subcommand, `build`
            // reaching no backend and so needing no account.
            Self::Event(cmd) => cmd.execute(printer, config_paths, account_name, backend),
            Self::Item(cmd) => cmd.execute(printer, config_paths, account_name, backend),

            #[cfg(feature = "caldav")]
            Self::Caldav(cmd) => {
                let client = build_caldav_client(config_paths, account_name)?;
                cmd.execute(printer, client)
            }
            #[cfg(feature = "gcal")]
            Self::Gcal(cmd) => {
                let client = build_gcal_client(config_paths, account_name)?;
                cmd.execute(printer, client)
            }
            #[cfg(feature = "pimdir")]
            Self::Pimdir(cmd) => {
                let backend = PimdirBackend::build(config_paths, account_name)?;
                cmd.execute(printer, backend)
            }
            #[cfg(feature = "vdir")]
            Self::Vdir(cmd) => {
                let client = build_vdir_client(config_paths, account_name)?;
                cmd.execute(printer, client)
            }

            Self::Account(cmd) => cmd.execute(printer, config_paths, account_name, backend),
            Self::Completion(cmd) => cmd.execute(printer, CalendulaCli::command()),
            Self::Manual(cmd) => cmd.execute(printer, CalendulaCli::command()),
            Self::JsonSchema(cmd) => cmd.execute(printer, json_schema::registry()),
        }
    }
}
