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
//! [`io_gcal`] the Google Calendar API v3 ones, [`io_vdir`] the local vdir
//! filesystem, [`io_pimdir`] the offline store and the sync engine that
//! fills it.
//!
//! [`io_http`] is the request and response state machines WebDAV and the REST
//! clients run on, `io_pim_discovery` the RFC 6764 CalDAV discovery, and
//! `ical` the iCalendar parser and editor.
//!
//! The `pimalaya_cli`, `pimalaya_config` and `pimalaya_stream` toolkits supply
//! the clap scaffolding, the TOML loading and secrets, and the blocking
//! transport. Every coroutine lives in those libraries and calendula
//! implements none: all real I/O is concentrated there.
//!
//! ## The cross-protocol layer calendula owns
//!
//! There is no aggregator crate between the CLI and the io-* libraries.
//! calendula owns its own cross-protocol types
//! ([`shared::calendar::Calendar`], [`shared::item::CalendarItem`]) and its
//! own dispatcher ([`shared::client::CalendarClient`]), an enum holding
//! exactly one backend.
//!
//! Each protocol module carries a backend submodule adapting that library to
//! the unified surface.
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
//! ### The projected backend
//!
//! Three backends speak iCalendar natively and store the bytes verbatim.
//! Google does not: it holds a JSON event and exposes no per-event iCalendar
//! representation.
//!
//! So io-gcal's `ical` feature synthesizes [`shared::item::CalendarItem`]'s
//! contents on read and re-projects them on write, stashing whatever the
//! projection neither manages nor mints so it survives a round-trip. The
//! policy is written down in cairn/spec/projection.md.
//!
//! ## Three command families
//!
//! The command tree ([`cli`]) is split into three groups, in this order. The
//! unified API (`calendar`, `event`, `item`) is the portable surface, where
//! an operation means the same whichever backend serves the account.
//!
//! The protocol-specific APIs (`caldav`, `gcal`, `pimdir`, `vdir`) each expose
//! what only one backend has, gated behind its own cargo feature. The meta
//! commands (`account`, `completions`, `manuals`) inspect the configuration
//! and what its backends serve, and generate the shell integration.
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
//! calendula never writes a configuration: a person writes it from the
//! documented sample. A person running calendula without one meets the
//! welcome ([`cli::welcome`], pimalaya-cli's in calendula's words), which
//! creates it from the sample shipped beside the binary or downloads it; a
//! script or a JSON caller gets the error instead. A bare `calendula` prints
//! the help when a configuration exists.
//!
//! ## Output
//!
//! All data and errors go to stdout through the printer, with `--json`
//! switching every command to JSON; stderr carries the logs, the prompts and
//! the welcome. A command returns a `Serialize + Display` value rather than
//! printing inline.
//!
//! Text output is plain text, with no border, no colour and nothing sized
//! to the terminal: a list of short rows prints as aligned columns, a wide
//! one as records the way `git log` prints commits, and a record as aligned
//! `Label: value` lines, every string a server or a file gave defused first.
//! `event list` prints its events one line each between a header naming the
//! account and the calendar and a footer saying where the page stands.
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
#[cfg(feature = "msgraph")]
mod msgraph;
#[cfg(feature = "pimdir")]
mod pimdir;
mod shared;
#[cfg(feature = "vdir")]
mod vdir;

use std::{path::PathBuf, process};

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{
    clap::args::LogFlags,
    error::{EXIT_FAILURE, ErrorReport},
    log::Logger,
    printer::StdoutPrinter,
    welcome::{is_wanted, shipped_sample},
};
use pimalaya_config::toml::ConfigMissingError;
use simd_cpu::SimdCpu;

use crate::{
    backend::Backend,
    cli::{Cli, Command},
    error::ErrorCode,
};

fn main() {
    // NOTE: the base64 and quoted-printable codecs read the installed CPU
    // token, the portable baseline until one is installed.
    let _ = SimdCpu::install_detected();

    let cli = Cli::parse();
    let mut printer = StdoutPrinter::new(&cli.json);

    let config_paths = cli.config.paths;
    let account = cli.account.name.as_deref();
    let result = execute(
        cli.cmd,
        &cli.log,
        &config_paths,
        account,
        cli.backend,
        &mut printer,
    );

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

/// Runs the subcommand, or meets a bare `calendula`.
fn execute(
    cmd: Option<Command>,
    log: &LogFlags,
    config_paths: &[PathBuf],
    account: Option<&str>,
    backend: Backend,
    printer: &mut StdoutPrinter,
) -> Result<()> {
    Logger::try_init(log)?;

    let Some(cmd) = cmd else {
        return cli::meet_bare_invocation(config_paths);
    };

    cmd.execute(printer, config_paths, account, backend)
}
