//! # Calendula
//!
//! A CLI to manage calendars, and the top of the Pimalaya stack: no library
//! target, no protocol or storage logic of its own, a thin shell driving the
//! sans-I/O libraries below and owning the cross-protocol abstraction over
//! them.
//!
//! This header is the architecture document of the crate. The living
//! specification, the in-flight proposals and the landed history are in the
//! cairn folder, one file per capability under cairn/spec.
//!
//! ## The stack below
//!
//! [`io_webdav`] carries the CalDAV coroutines (RFC 4791 over RFC 4918),
//! [`io_gcal`] the Google Calendar API v3 ones, [`io_msgraph`] the
//! Microsoft Graph calendar ones, [`io_vdir`] the local vdir filesystem,
//! [`io_pimdir`] the offline store a sync engine fills, its reader, its
//! producer queueing writes and its search index.
//!
//! [`io_http`] is the request and response state machines WebDAV and the REST
//! clients run on, `io_pim_discovery` the RFC 6764 CalDAV discovery behind
//! `caldav.discover`, and `ical` the iCalendar parser and editor.
//!
//! The `pimalaya_cli`, `pimalaya_config` and `pimalaya_stream` toolkits supply
//! the clap scaffolding, the printer and its plain text shapes, the welcome,
//! the REPL and MCP core; the TOML loading, the accounts and the secrets;
//! and the blocking transport with its TLS providers. Every coroutine lives
//! in those libraries and calendula implements none: all real I/O is
//! concentrated there.
//!
//! ## The cross-protocol layer calendula owns
//!
//! There is no aggregator crate between the CLI and the io-* libraries.
//! calendula owns its own cross-protocol types
//! ([`shared::calendar::Calendar`], [`shared::item::CalendarItem`]) and its
//! own dispatcher ([`shared::client::CalendarClient`]), an enum holding
//! exactly one backend.
//!
//! Five backends sit behind it, each behind its own cargo feature: caldav
//! over io-webdav, gcal over io-gcal, msgraph over io-msgraph, vdir over
//! io-vdir and pimdir over io-pimdir. Each protocol module carries a backend
//! submodule adapting that library to the unified surface.
//!
//! ### The capability union
//!
//! The unified API is the union of what the backends serve, not their
//! least common denominator. [`capability`] declares, in one table per
//! backend compiled into every build, each unified capability at `full`,
//! `partial`, `server` or `none`, named after pimdir STORAGE Annex B
//! without its domain word (`item.add`, `item.remove.if-match`,
//! `online-meeting`).
//!
//! A unified command requires the capabilities its verb and its flags
//! reach before anything connects, so one the backend does not serve fails
//! with the `capability-missing` code; the same tables write each unified
//! command's `Backends:` help footer and `account capability`. The
//! guardrail is one meaning per verb: a backend declares a capability only
//! when it gives the result every other declarer gives, and leaves another
//! result to its protocol-specific API.
//!
//! ### The projected backends
//!
//! CalDAV, vdir and pimdir speak iCalendar natively and store the bytes
//! verbatim. Google Calendar and Microsoft Graph do not: each holds a JSON
//! event and exposes no per-event iCalendar representation.
//!
//! So the `ical` features of io-gcal and io-msgraph synthesize
//! [`shared::item::CalendarItem`]'s contents on read and re-project them on
//! write, a recurring series and its exceptions folded into one item,
//! stashing whatever the projection neither manages nor mints so it
//! survives a round-trip. The policy is written down in
//! cairn/spec/projection.md, Graph's limits in cairn/spec/backends.md.
//!
//! ## Three command families
//!
//! The command tree ([`cli`]) is split into three groups, in this order. The
//! unified API (`calendar`, `event`, `item`) is the portable surface, where
//! an operation means the same whichever backend serves the account.
//!
//! The protocol-specific APIs (`caldav`, `gcal`, `pimdir`, `vdir`) each expose
//! what only one backend has, gated behind its own cargo feature. The meta
//! commands (`account`, `repl`, `mcp`, `completion`, `manual`,
//! `json-schema`) inspect the configuration and what its backends serve,
//! run the sessions, and generate the shell integration and the schemas.
//!
//! ### Events against item
//!
//! calendula is the Pimalaya CLI for events: VTODO belongs to unfog and
//! VJOURNAL to journal, one CLI per iCalendar domain. A calendar collection
//! may still mix component kinds, so the unified API offers two kinds of view
//! over the same resources. `item` is the raw, unfiltered one: it lists,
//! reads, writes and deletes any iCalendar object by id, a to-do or a journal
//! entry included, leaving the bytes untouched.
//!
//! `event` is the projected one: it keeps VEVENT, renders summary and time
//! columns and draws a cal(1)-style agenda.
//!
//! Both share the `-k/--calendar` selector and the same item API; only the
//! rendering and the filter differ, so the projection adds no backend
//! operation.
//!
//! VFREEBUSY and VTIMEZONE get no family: the first is the answer to a query
//! rather than a stored resource, and the second defines the zones the others
//! reference.
//!
//! ## Backend selection
//!
//! The unified commands target the backend the global `--backend` flag picks
//! ([`backend::Backend`]). Its default, `auto`, takes the first
//! configured-and-compiled backend in calendula's priority order: vdir,
//! pimdir, CalDAV, gcal, msgraph.
//!
//! That order prefers a local read to a network round-trip, and a
//! protocol-standard server to a vendor API. It skips no backend for a
//! capability it lacks: every unified verb exchanges calendar or item ids
//! the others take, so all of them run on one backend, and a refusal names
//! the backend of the account serving what was asked. A named value pins
//! the command and bails when the account carries no matching block, while
//! the protocol-specific commands ignore the flag and build their own
//! client.
//!
//! ## Sessions
//!
//! Every command runs over a [`session`] `Session`, which builds the
//! unified client on first use and keeps it, dropping it when a command
//! fails on it; the protocol-specific commands build theirs each time. A
//! one-shot run uses a fresh session for its single command; the [`repl`]
//! keeps one across the lines of its stdin, so the client connects once
//! for the whole session. The [`mcp`] server keeps one the same way, each
//! tool call parsed back into a unified API command. Both run on
//! pimalaya-cli's shared core (its `repl` loop, its `mcp` protocol answers
//! and tool grammar), which claims stdin, so a command reading its input
//! there is refused under `stdin-taken`, and so is the composer, an editor
//! fighting the session for the terminal, and a session inside a session.
//!
//! The server lists a tool only when the account's backend serves its verb
//! ([`capability::serves`]), the writes under `--allow-write` alone, and
//! leaves what reaches people beyond the account behind `--allow-send` as
//! well: the scheduling arguments, and any write or delete of an item
//! naming an attendee, which the server or a sync may notify. That last
//! guard runs in the unified client itself, on the contents written and on
//! the stored copy, so it holds on every backend.
//!
//! ## Configuration
//!
//! Configuration is loaded by pimalaya-config from every existing canonical
//! path (the rc file last), or from the `-c` paths, one per flag, later
//! paths deep-merged on top of the first. pimalaya-config also takes the
//! account `-a` names or the default one, its typed errors naming what is
//! missing.
//!
//! The schema is multi-account: a top-level block plus named account blocks,
//! each carrying an optional sub-block per backend, and the global block is
//! folded under the selected account.
//!
//! calendula never writes a configuration: a person writes it from
//! config.sample.toml, one section per backend and provider, which the test
//! suite parses variant by variant and the packages ship in
//! share/doc/calendula. A person running calendula without one meets the
//! welcome ([`cli::welcome`], pimalaya-cli's in calendula's words), which
//! creates it from the shipped sample or downloads it; a script or a JSON
//! caller gets the error instead. A bare `calendula` prints the help.
//!
//! A key or a table calendula no longer reads (`todo`, `journal`, `table`,
//! the top-level `account`) is refused by name, saying where it went, rather
//! than ignored.
//!
//! ## Output
//!
//! All data and errors go to stdout through the printer, with `--json`
//! switching every command to JSON; stderr carries the logs, the prompts,
//! the welcome, the REPL's help in a JSON session, and the failures of the
//! MCP server, whose stdout is the protocol's. A command returns a
//! `Serialize + Display` value rather than printing inline.
//!
//! A failure prints through pimalaya-cli's `ErrorReport` with the stable
//! [`error::ErrorCode`] a caller acts on, `null` otherwise, and exits 1; a
//! result whose outcome is not a full success, a failed `account check`,
//! prints its report and exits 2. Every write prints a receipt, the item it
//! acted on and whether it waits for a sync.
//!
//! Text output is plain text, with no border, no colour and nothing sized
//! to the terminal: a list of short rows prints as aligned columns, a wide
//! one as records the way `git log` prints commits, and a record as aligned
//! `Label: value` lines, every string a server or a file gave defused first.
//! `event list` and `item list` print one line each between a header naming
//! the account and the calendar and a footer saying where the page stands;
//! their JSON names the next page as `nextPage`, which `--page-token` takes
//! back. A read prints the stored iCalendar defused of what could drive the
//! terminal, its JSON keeping the bytes.
//!
//! Each command's doc comment is its help text, which makes `calendula
//! <command> --help` the usage reference for both humans and agents, and is
//! why the README documents no per-command usage.

mod account;
mod backend;
#[cfg(feature = "caldav")]
mod caldav;
mod capability;
mod cli;
mod config;
mod error;
#[cfg(feature = "gcal")]
mod gcal;
mod json_schema;
mod mcp;
#[cfg(feature = "msgraph")]
mod msgraph;
#[cfg(feature = "pimdir")]
mod pimdir;
mod repl;
mod session;
mod shared;
#[cfg(feature = "vdir")]
mod vdir;

use std::process;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{
    error::{EXIT_FAILURE, ErrorReport},
    log::Logger,
    printer::StdoutPrinter,
    welcome::{is_wanted, shipped_sample},
};
use pimalaya_config::toml::ConfigMissingError;
use simd_cpu::SimdCpu;

use crate::{
    cli::Cli,
    error::ErrorCode,
    session::{Session, SessionAccount},
};

fn main() {
    // NOTE: the base64 and quoted-printable codecs read the installed CPU
    // token, the portable baseline until one is installed.
    let _ = SimdCpu::install_detected();

    let cli = Cli::parse();
    let mut printer = StdoutPrinter::new(&cli.json);
    let config_paths = cli.config.paths.clone();
    let result = execute(cli, &mut printer);

    // NOTE: a person missing a configuration meets the welcome, a script
    // or a JSON caller the error.
    if let Err(err) = &result
        && let Some(missing) = err.downcast_ref::<ConfigMissingError>()
        && is_wanted(&printer)
    {
        let sample = shipped_sample("calendula");
        let welcome = cli::welcome(&missing.path, &config_paths, sample.as_deref());
        eprint!("{welcome}");
        process::exit(EXIT_FAILURE);
    }

    ErrorReport::<ErrorCode>::eval_coded(&mut printer, result);
}

/// Runs the subcommand over a fresh session, or meets a bare `calendula`.
fn execute(cli: Cli, printer: &mut StdoutPrinter) -> Result<()> {
    Logger::try_init(&cli.log)?;

    let account = SessionAccount {
        config_paths: &cli.config.paths,
        name: cli.account.name.as_deref(),
        backend: cli.backend,
    };

    let Some(cmd) = cli.cmd else {
        return cli::meet_bare_invocation(account.config_paths);
    };

    let mut session = Session {
        account,
        ..Default::default()
    };

    cmd.execute(printer, &mut session)
}
