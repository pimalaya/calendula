//! # CLI
//!
//! The command tree calendula serves and its single dispatch point, plus
//! the account resolution and the first-run welcome every command starts
//! from.

use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use pimalaya_cli::{
    clap::{
        args::{AccountFlag, ConfigFlags, JsonFlag, LogFlags},
        commands::{
            completion::CompletionCommand, json_schema::JsonSchemaCommand, manual::ManualCommand,
        },
    },
    footer, long_version,
    printer::Printer,
    welcome::Welcome,
};
use pimalaya_config::toml::{ConfigMissingError, TomlConfig};

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
    config::{AccountConfig, Config},
    json_schema,
    shared::{
        calendar::cli::CalendarCommand, client::CalendarClient, event::cli::EventCommand,
        item::cli::ItemCommand,
    },
};

/// The sample configuration as a raw file, for `curl` to fetch.
const SAMPLE_RAW_URL: &str =
    "https://raw.githubusercontent.com/pimalaya/calendula/master/config.sample.toml";

/// Top-level command-line interface parser.
#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(author, version, about)]
#[command(long_version = long_version!())]
#[command(after_help = footer!())]
#[command(propagate_version = true, infer_subcommands = true)]
pub struct Cli {
    /// The command to run, a bare `calendula` showing this help.
    #[command(subcommand)]
    pub cmd: Option<Command>,
    #[command(flatten)]
    pub config: ConfigFlags,
    #[command(flatten)]
    pub account: AccountFlag,
    /// Force a specific backend for the cross-protocol commands.
    ///
    /// The unified commands (`calendar`, `event`, `item`), `account
    /// check` and `account capability` read it: the protocol-specific
    /// subcommands each already name their backend.
    ///
    /// With `auto`, the first configured backend wins, in calendula's
    /// priority order (vdir, pimdir, caldav, gcal, msgraph), whatever
    /// the command needs: one it does not serve is refused, naming the
    /// backend of the account that does. Any other value pins that one,
    /// and the command bails when the account carries no matching block.
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
pub enum Command {
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

/// Resolves the account a command runs against, returning the leftover
/// global config, the account name and its config.
///
/// A missing configuration fails with a [`ConfigMissingError`] naming the
/// path looked at, which is where `-c` pointed, so a mistyped path shows
/// up as itself and `main` can meet a person with the welcome.
pub fn resolve_account(
    config_paths: &[PathBuf],
    account_name: Option<&str>,
) -> Result<(Config, String, AccountConfig)> {
    let mut config = Config::load(config_paths)?;
    let (name, account_config) = config.take_account(account_name)?;
    Ok((config, name, account_config))
}

/// Meets a bare `calendula`: the help when a configuration exists, the
/// missing configuration otherwise, which a person meets as the welcome.
pub fn meet_bare_invocation(config_paths: &[PathBuf]) -> Result<()> {
    // NOTE: a broken file counts as a configuration, its error left to
    // the first command reading it.
    if let Err(err) = Config::load(config_paths)
        && err.is::<ConfigMissingError>()
    {
        return Err(err);
    }

    Cli::command().print_help()?;
    Ok(())
}

/// The welcome a person meets when no configuration exists at `path`:
/// the commands creating it from `sample`, the shipped sample
/// configuration, or downloading it without one, then `event list`.
pub fn welcome<'a>(
    path: &'a Path,
    config_paths: &'a [PathBuf],
    sample: Option<&'a Path>,
) -> Welcome<'a> {
    Welcome {
        bin: "calendula",
        intro: "Welcome to Calendula, the CLI to manage calendars.",
        path,
        config_paths,
        sample,
        sample_url: SAMPLE_RAW_URL,
        next: "Edit it and uncomment the options you need, then list your events:",
        command: "event list",
    }
}

impl Command {
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
            Self::Completion(cmd) => cmd.execute(printer, Cli::command()),
            Self::Manual(cmd) => cmd.execute(printer, Cli::command()),
            Self::JsonSchema(cmd) => cmd.execute(printer, json_schema::registry()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{env, fs, path::PathBuf, process};

    use clap::Parser;
    use pimalaya_config::toml::{AccountError, ConfigMissingError};
    use tempfile::TempDir;

    use crate::cli::{Cli, resolve_account};

    /// A directory that is never created, so the paths below it keep the
    /// spelling they were given instead of being canonicalized.
    fn absent_dir() -> PathBuf {
        env::temp_dir().join(format!("calendula-config-paths-{}", process::id()))
    }

    fn paths(args: &[&str]) -> Vec<PathBuf> {
        Cli::try_parse_from([&["calendula"], args].concat())
            .expect("parse")
            .config
            .paths
    }

    /// Writes `toml` as the only configuration file of a fresh directory.
    fn config(toml: &str) -> (TempDir, Vec<PathBuf>) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(&path, toml).unwrap();
        (dir, vec![path])
    }

    /// A value is one path whatever it holds, `$PATH`'s delimiters
    /// included; several paths take one `-c` each, in order.
    #[test]
    fn a_delimited_config_value_stays_one_path() {
        let dir = absent_dir();
        let given = dir.join("one.toml:two.toml;three.toml");
        let args = ["-c", given.to_str().unwrap(), "account", "list"];
        assert_eq!(paths(&args), [given]);

        let (one, two) = (dir.join("one.toml"), dir.join("two.toml"));
        let (one_str, two_str) = (one.to_str().unwrap(), two.to_str().unwrap());
        let args = ["-c", two_str, "-c", one_str, "account", "list"];
        assert_eq!(paths(&args), [two, one]);
    }

    /// Text output is sized to nothing, so no list takes a width.
    #[test]
    fn a_max_width_is_refused() {
        for list in [
            &["calendar", "list"][..],
            &["event", "list"],
            &["event", "find", "1@example.org"],
            &["item", "list"],
        ] {
            let args = [&["calendula"][..], list, &["-w", "80"]].concat();
            assert!(Cli::try_parse_from(args).is_err(), "{list:?}");
        }
    }

    /// `main` meets a person with the welcome by downcasting this error,
    /// so it has to come typed, naming the path `-c` gave.
    #[test]
    fn a_missing_configuration_is_typed_for_the_welcome() {
        let paths = [absent_dir().join("config.toml")];
        let err = resolve_account(&paths, None).unwrap_err();
        let missing = err.downcast_ref::<ConfigMissingError>().unwrap();
        assert_eq!(missing.path, paths[0]);
    }

    #[test]
    fn an_unknown_account_lists_the_held_ones() {
        let (_dir, paths) = config("[accounts.work]\n[accounts.perso]\n");
        let err = resolve_account(&paths, Some("nope")).unwrap_err();

        match err.downcast_ref::<AccountError>() {
            Some(AccountError::Missing { name, names }) => {
                assert_eq!(name, "nope");
                assert_eq!(names, &["perso", "work"]);
            }
            _ => panic!("{err:?}"),
        }

        let message = err.to_string();
        assert!(message.ends_with("holds: perso, work"), "{message}");
    }

    /// The config layers make the case easy to reach, an rc file marking
    /// one account the default while the platform file marks another.
    #[test]
    fn several_default_accounts_are_refused_naming_them() {
        let account = |name: &str| format!("[accounts.{name}]\ndefault = true\n");
        let (_dir, paths) = config(&(account("work") + &account("perso")));
        let err = resolve_account(&paths, None).unwrap_err();

        match err.downcast_ref::<AccountError>() {
            Some(AccountError::SeveralDefaults(names)) => assert_eq!(names, &["perso", "work"]),
            _ => panic!("{err:?}"),
        }

        let (_, name, _) = resolve_account(&paths, Some("work")).unwrap();
        assert_eq!(name, "work");
    }
}
