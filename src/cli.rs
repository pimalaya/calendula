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
    error::with_code,
    footer, long_version,
    mcp::exit_on_failure,
    printer::Printer,
    welcome::{Welcome, is_wanted},
};
use pimalaya_config::toml::{ConfigMissingError, TomlConfig};

#[cfg(feature = "caldav")]
use crate::caldav::{
    cli::CaldavCommand,
    client::{build_caldav_client, caldav_config},
};
#[cfg(feature = "gcal")]
use crate::gcal::{cli::GcalCommand, client::build_gcal_client};
#[cfg(feature = "msgraph")]
use crate::msgraph::{cli::MsgraphCommand, client::build_msgraph_client};
#[cfg(feature = "pimdir")]
use crate::pimdir::{backend::PimdirBackend, cli::PimdirCommand};
#[cfg(feature = "vdir")]
use crate::vdir::{cli::VdirCommand, client::build_vdir_client};
use crate::{
    account::cli::AccountCommand,
    backend::{self, Backend},
    config::{AccountConfig, Config},
    error::{self, ErrorCode},
    json_schema,
    mcp::McpCommand,
    repl::ReplCommand,
    session::Session,
    shared::{calendar::cli::CalendarCommand, event::cli::EventCommand, item::cli::ItemCommand},
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
    /// Force a specific backend for cross-protocol commands.
    ///
    /// Only the unified commands read it (`calendar`, `event`, `item`,
    /// `account check`, `account capability`, and `repl` and `mcp` for
    /// their session), the protocol-specific ones always using their own
    /// backend. `auto`, the default, picks the first configured backend,
    /// whatever the command needs: a command it does not serve is refused
    /// naming the backend of the account that does, and a backend this
    /// build leaves out naming the cargo feature bringing it back. An
    /// explicit value bails when the account has no such block, or the
    /// command no such implementation.
    #[arg(short, long, global = true, default_value_t, value_parser = backend::parser())]
    pub backend: Backend,
    #[command(flatten)]
    pub json: JsonFlag,
    #[command(flatten)]
    pub log: LogFlags,
}

/// Every command calendula serves: the unified API, the
/// protocol-specific escape hatches, then the meta commands.
///
/// The enum carries the version too, so a `--version` line of a session
/// prints it as the command line does.
#[derive(Debug, Subcommand)]
#[command(version, long_version = long_version!())]
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
    #[cfg(feature = "msgraph")]
    #[command(subcommand)]
    Msgraph(MsgraphCommand),
    #[cfg(feature = "gcal")]
    #[command(subcommand)]
    Gcal(GcalCommand),
    #[cfg(feature = "pimdir")]
    #[command(subcommand)]
    Pimdir(PimdirCommand),
    #[cfg(feature = "vdir")]
    #[command(subcommand)]
    Vdir(VdirCommand),
    Repl(ReplCommand),
    Mcp(McpCommand),
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
    /// Runs the command over `session`, whose account the protocol
    /// commands resolve and whose unified client the unified ones keep.
    ///
    /// A server's refusal or an account pimalaya-config could not take
    /// carries its code to the failure object (pimalaya-cli's `with_code`),
    /// whoever prints it: the one-shot run, the REPL or the MCP server.
    pub fn execute(self, printer: &mut impl Printer, session: &mut Session) -> Result<()> {
        self.run(printer, session)
            .map_err(|err| with_code(err, error::code))
    }

    /// Runs the command, its failure as the backend gave it.
    fn run(self, printer: &mut impl Printer, session: &mut Session) -> Result<()> {
        let account = session.account;
        let (config_paths, account_name) = (account.config_paths, account.name);

        match self {
            Self::Calendar(cmd) => session.run_unified(printer, |p, client| cmd.execute(p, client)),
            Self::Event(cmd) => cmd.execute(printer, session),
            Self::Item(cmd) => cmd.execute(printer, session),

            #[cfg(feature = "caldav")]
            Self::Caldav(cmd) => {
                let config = caldav_config(config_paths, account_name)?;
                cmd.check()?;
                let client = build_caldav_client(&config)?;
                cmd.execute(printer, client)
            }
            #[cfg(feature = "msgraph")]
            Self::Msgraph(cmd) => {
                let client = build_msgraph_client(config_paths, account_name)?;
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

            Self::Repl(cmd) => cmd.execute(&*printer, session),
            Self::Mcp(cmd) => {
                // NOTE: stdout carries protocol messages only, so a failure
                // goes to stderr, a person missing a configuration aside.
                match cmd.execute(session) {
                    Err(err) if err.is::<ConfigMissingError>() && is_wanted(&*printer) => Err(err),
                    result => {
                        exit_on_failure::<ErrorCode, _>(
                            result.map_err(|err| with_code(err, error::code)),
                        );
                        Ok(())
                    }
                }
            }
            Self::Account(cmd) => cmd.execute(printer, config_paths, account_name, account.backend),
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

    /// An option after a `pimdir search` or `pimdir count` query is read
    /// as the option, never as a query word, `--json` included; a word
    /// starting with `-` passes after `--`, and is refused before it.
    #[cfg(feature = "pimdir")]
    #[test]
    fn an_option_after_a_query_is_an_option() {
        use crate::{
            cli::Command,
            pimdir::cli::PimdirCommand::{Count, Search},
        };

        let parse = |args: &[&str]| Cli::try_parse_from([&["calendula"][..], args].concat());

        let cli = parse(&["pimdir", "search", "x", "--page-size", "3", "--json"]).unwrap();
        assert!(cli.json.enabled);
        let Some(Command::Pimdir(Search(search))) = cli.cmd else {
            panic!("not a search");
        };
        assert_eq!(search.query, ["x"]);
        assert_eq!(search.page_size, Some(3));

        let cli = parse(&["pimdir", "count", "budget", "-k", "work"]).unwrap();
        let Some(Command::Pimdir(Count(count))) = cli.cmd else {
            panic!("not a count");
        };
        assert_eq!(count.query, ["budget"]);
        assert_eq!(count.calendar.as_deref(), Some("work"));

        let cli = parse(&["pimdir", "search", "-s", "5", "--", "budget", "-lisbon"]).unwrap();
        let Some(Command::Pimdir(Search(search))) = cli.cmd else {
            panic!("not a search");
        };
        assert_eq!(search.query, ["budget", "-lisbon"]);
        assert_eq!(search.page_size, Some(5));

        assert!(parse(&["pimdir", "search", "budget", "-lisbon"]).is_err());
    }

    /// Text output is sized to nothing, so no list takes a width.
    #[test]
    fn a_max_width_is_refused() {
        for list in [
            &["calendar", "list"][..],
            &["event", "list"],
            &["event", "search", "1@example.org"],
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

#[cfg(all(
    test,
    feature = "caldav",
    feature = "gcal",
    feature = "msgraph",
    feature = "vdir"
))]
mod namespace_tests {
    use clap::{CommandFactory, Parser};

    use crate::{
        caldav::{cli::CaldavCommand, report::cli::CaldavReportCommand},
        cli::{Cli, Command},
        gcal::{cli::GcalCommand, events::GcalEventsCommand},
        msgraph::{
            calendars::cli::MsgraphCalendarsCommand, cli::MsgraphCommand,
            events::cli::MsgraphEventsCommand,
        },
        vdir::{cli::VdirCommand, item::cli::VdirItemCommand},
    };

    /// The command a line runs, which must parse.
    fn parse(line: &str) -> Command {
        let args = line.split_whitespace();
        let cli = Cli::try_parse_from(["calendula"].into_iter().chain(args));
        cli.unwrap_or_else(|err| panic!("{line}: {err}"))
            .cmd
            .unwrap()
    }

    /// Whether `path` names a command the help, the manual and the
    /// completions hide.
    fn hidden(path: &[&str]) -> bool {
        let mut command = Cli::command();

        for name in path {
            command = command
                .find_subcommand(name)
                .unwrap_or_else(|| panic!("{path:?}"))
                .clone();
        }

        command.is_hide_set()
    }

    /// Every method of `caldav` parses, with the ids positional.
    #[test]
    fn the_caldav_methods_parse() {
        use CaldavCommand::*;

        assert!(matches!(
            parse("caldav discover"),
            Command::Caldav(Discover(_))
        ));
        let Command::Caldav(Propfind(cmd)) = parse("caldav propfind cal") else {
            panic!()
        };
        assert_eq!(cmd.calendar_id.as_deref(), Some("cal"));
        let Command::Caldav(Proppatch(cmd)) = parse("caldav proppatch cal -n Work -C #ff0000")
        else {
            panic!()
        };
        assert_eq!(cmd.name.as_deref(), Some("Work"));
        assert!(matches!(
            parse("caldav mkcalendar cal --component VEVENT"),
            Command::Caldav(Mkcalendar(_))
        ));
        assert!(matches!(
            parse("caldav report query cal --match SUMMARY standup --from 2026-11-16"),
            Command::Caldav(Report(CaldavReportCommand::Query(_)))
        ));
        assert!(matches!(
            parse("caldav report multiget cal e1.ics e2.ics"),
            Command::Caldav(Report(CaldavReportCommand::Multiget(_)))
        ));
        assert!(matches!(
            parse("caldav report sync cal --sync-token t"),
            Command::Caldav(Report(CaldavReportCommand::Sync(_)))
        ));
        assert!(matches!(
            parse("caldav report raw cal <x/>"),
            Command::Caldav(Report(CaldavReportCommand::Raw(_)))
        ));
        assert!(matches!(
            parse("caldav get cal e1.ics"),
            Command::Caldav(Get(_))
        ));
        assert!(matches!(
            parse("caldav put cal e1.ics - --if-match e1"),
            Command::Caldav(Put(_))
        ));
        let Command::Caldav(Delete(cmd)) = parse("caldav delete cal e1.ics") else {
            panic!()
        };
        assert_eq!(cmd.item_id.as_deref(), Some("e1.ics"));
    }

    /// The former `caldav` names run the methods they became, flags
    /// included, hidden from the help.
    #[test]
    fn the_former_caldav_names_stay_hidden_aliases() {
        use CaldavCommand::*;

        for line in ["caldav list", "caldav ls"] {
            let Command::Caldav(Propfind(cmd)) = parse(line) else {
                panic!("{line}")
            };
            assert_eq!(cmd.calendar_id, None, "{line}");
        }

        let line = "caldav create cal --display-name Work -D Team -C #ff0000";
        let Command::Caldav(Mkcalendar(cmd)) = parse(line) else {
            panic!()
        };
        assert_eq!(cmd.id, "cal");
        assert_eq!(cmd.name.as_deref(), Some("Work"));
        assert_eq!(cmd.description.as_deref(), Some("Team"));

        let Command::Caldav(Delete(cmd)) = parse("caldav delete cal") else {
            panic!()
        };
        assert_eq!(cmd.item_id, None);

        let caldav = Cli::command();
        let caldav = caldav.find_subcommand("caldav").unwrap();
        let propfind = caldav.find_subcommand("propfind").unwrap();
        assert!(propfind.get_visible_aliases().next().is_none());
        let mkcalendar = caldav.find_subcommand("mkcalendar").unwrap();
        assert!(mkcalendar.get_visible_aliases().next().is_none());
    }

    /// The Events methods live under `gcal events`, the former flat names
    /// running them hidden.
    #[test]
    fn the_gcal_event_methods_move_under_events() {
        use GcalCommand as Gcal;
        use GcalEventsCommand as Events;

        assert!(matches!(
            parse("gcal events instances E1 --from 2026-11-16"),
            Command::Gcal(Gcal::Events(Events::Instances(_)))
        ));
        assert!(matches!(
            parse("gcal event move E1 work"),
            Command::Gcal(Gcal::Events(Events::Move(_)))
        ));
        assert!(matches!(
            parse("gcal events quick-add Lunch"),
            Command::Gcal(Gcal::Events(Events::QuickAdd(_)))
        ));

        assert!(matches!(
            parse("gcal instances E1"),
            Command::Gcal(Gcal::Instances(_))
        ));
        assert!(matches!(
            parse("gcal instance E1"),
            Command::Gcal(Gcal::Instances(_))
        ));
        assert!(matches!(
            parse("gcal move E1 work"),
            Command::Gcal(Gcal::Move(_))
        ));
        assert!(matches!(
            parse("gcal quick-add Lunch"),
            Command::Gcal(Gcal::QuickAdd(_))
        ));

        for name in ["instances", "move", "quick-add"] {
            assert!(hidden(&["gcal", name]), "{name}");
            assert!(!hidden(&["gcal", "events", name]), "{name}");
        }
    }

    /// The Graph resources parse, their singular hidden.
    #[test]
    fn the_msgraph_resources_parse() {
        use MsgraphCalendarsCommand as Calendars;
        use MsgraphCommand as Graph;
        use MsgraphEventsCommand as Events;

        assert!(matches!(
            parse("msgraph calendars list -s 5"),
            Command::Msgraph(Graph::Calendars(Calendars::List(_)))
        ));
        assert!(matches!(
            parse("msgraph calendar rename C1 Work"),
            Command::Msgraph(Graph::Calendars(Calendars::Rename(_)))
        ));
        assert!(matches!(
            parse("msgraph events list -k C1 --filter x"),
            Command::Msgraph(Graph::Events(Events::List(_)))
        ));
        assert!(matches!(
            parse("msgraph event rm E1"),
            Command::Msgraph(Graph::Events(Events::Delete(_)))
        ));
        assert!(matches!(
            parse("msgraph events delta --from 2026-11-16 --to 2026-11-22"),
            Command::Msgraph(Graph::Events(Events::Delta(_)))
        ));
        assert!(matches!(
            parse("msgraph events delta --delta-link https://graph.example.org/x"),
            Command::Msgraph(Graph::Events(Events::Delta(_)))
        ));
        assert!(matches!(
            parse("msgraph profile get"),
            Command::Msgraph(Graph::Profile(_))
        ));
        assert!(matches!(
            parse("msgraph request get me"),
            Command::Msgraph(Graph::Request(_))
        ));

        let window = "calendula msgraph events delta --from 2026-11-16";
        assert!(Cli::try_parse_from(window.split_whitespace()).is_err());
    }

    /// The vdir items parse, under `item` and `items`.
    #[test]
    fn the_vdir_items_parse() {
        use VdirItemCommand as Item;

        for line in ["vdir item list cal", "vdir items ls cal"] {
            assert!(
                matches!(parse(line), Command::Vdir(VdirCommand::Item(Item::List(_)))),
                "{line}"
            );
        }

        assert!(matches!(
            parse("vdir item create cal --kind ical -"),
            Command::Vdir(VdirCommand::Item(Item::Create(_)))
        ));
        assert!(matches!(
            parse("vdir item move cal work e1"),
            Command::Vdir(VdirCommand::Item(Item::Move(_)))
        ));
        assert!(matches!(
            parse("vdir ls"),
            Command::Vdir(VdirCommand::List(_))
        ));
    }
}
