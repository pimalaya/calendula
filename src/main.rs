//! # calendula
//!
//! A CLI to manage calendars, and the top layer of the Pimalaya stack:
//! it has no library target and writes no protocol or storage logic of
//! its own. It is a thin shell driving the sans-I/O libraries below it,
//! and owning the cross-protocol abstraction over them.
//!
//! This header is the architecture document of the crate. The living
//! specification, the in-flight proposals and the landed history are in
//! the cairn folder, one file per capability under cairn/spec.
//!
//! ## The stack below
//!
//! [`io_webdav`] carries the CalDAV protocol coroutines (RFC 4791 over
//! RFC 4918), [`io_gcal`] the Google Calendar API v3 coroutines,
//! [`io_vdir`] the local vdir filesystem coroutines, [`io_pimdir`] and
//! [`io_replica`] the offline store and the sync engine model behind
//! it, [`io_http`] the request and response state machines WebDAV and
//! the REST clients run on, and `io_pim_discovery` the RFC 6764 CalDAV
//! discovery. `ical` parses and edits the iCalendar bytes. The
//! `pimalaya_cli`, `pimalaya_config` and `pimalaya_stream` toolkits
//! supply the clap scaffolding, the TOML loading and secrets, and the
//! blocking transport.
//!
//! Every coroutine lives in those libraries and calendula implements
//! none. All real I/O is concentrated there; calendula orchestrates and
//! renders.
//!
//! ## The cross-protocol layer calendula owns
//!
//! There is no aggregator crate between the CLI and the io-* libraries.
//! calendula owns its own least-common-denominator types
//! ([`shared::calendar::Calendar`], [`shared::item::CalendarItem`])
//! and its own dispatcher ([`shared::client::CalendarClient`]), an enum
//! holding exactly one backend. Each protocol module carries a backend
//! submodule adapting that library to the shared surface, so an
//! operation only reaches the shared API when every backend can serve
//! it.
//!
//! ### The projected backend
//!
//! Three backends speak iCalendar natively and store the bytes
//! verbatim. Google does not: it holds a JSON event and exposes no
//! per-event iCalendar representation, so [`gcal::project`] synthesizes
//! [`shared::item::CalendarItem`]'s contents on read and re-projects
//! them on write, stashing whatever the projection neither manages nor
//! mints so it survives a round-trip. The policy is written down in
//! cairn/spec/projection.md.
//!
//! ## Three command families
//!
//! The command tree ([`cli`]) is split into three groups, in this
//! order. The shared API (`calendar`, `event`, `todo`, `journal`,
//! `item`) is the portable surface, where every operation works the
//! same whichever backend serves the account. The protocol-specific
//! APIs (`caldav`, `gcal`, `pimdir`, `vdir`) each expose what only one
//! backend has, gated behind its own cargo feature. The meta commands
//! (`account`, `completions`, `manuals`) inspect the configuration and
//! generate the shell integration.
//!
//! ### The component families against item
//!
//! A calendar collection mixes component kinds, so the shared API
//! offers two kinds of view over the same resources. `item` is the raw,
//! unfiltered one: it lists, reads, writes and deletes any iCalendar
//! object by id, leaving the bytes untouched. The component families
//! are the projected ones, one per kind a calendar stores: `event`
//! (VEVENT) renders summary and time columns and draws a cal(1)-style
//! agenda, `todo` (VTODO) renders due date, status, priority and
//! completion, `journal` (VJOURNAL) renders a dated note. Each keeps
//! its own kind and drops the rest.
//!
//! They all share the `-k/--calendar` selector and the same item API;
//! only the rendering and the filter differ, so a component family
//! costs a projection and a table and adds no backend operation.
//! VFREEBUSY and VTIMEZONE get no family: the first is the answer to a
//! query rather than a stored resource, and the second defines the
//! zones the others reference.
//!
//! ## Backend selection
//!
//! The shared commands target the backend the global `--backend` flag
//! picks ([`backend::Backend`]). Its default, `auto`, takes the first
//! configured-and-compiled backend in calendula's priority order (vdir,
//! pimdir, CalDAV, gcal), preferring a local read to a network
//! round-trip and a protocol-standard server to a vendor API. A named
//! value pins the command and bails when the account carries no
//! matching block. The protocol-specific commands ignore the flag and
//! build their own client.
//!
//! ## Configuration and the wizard
//!
//! Configuration is loaded by pimalaya-config from the first existing
//! canonical path, or from `-c` / `CALENDULA_CONFIG`, with later paths
//! deep-merged on top of the first. The schema is multi-account: a
//! top-level block plus named account blocks, each carrying an optional
//! sub-block per backend, and the global block is folded under the
//! selected account.
//!
//! `calendula configure` runs the wizard ([`wizard::discover`] builds
//! the account, [`wizard::configure`] places it). One prompt takes an
//! address, a server URL or a folder path, and its shape orients the
//! flow. The account is saved to a configuration that does not exist,
//! appended to one that does, or printed when stdout is redirected.
//!
//! The wizard is also offered where nothing can happen without a
//! configuration: a bare `calendula` finding none, and a command needing
//! an account finding none. A bare `calendula` finding one prints the
//! help instead.
//!
//! ## Output
//!
//! All data and errors go to stdout through the printer, with `--json`
//! switching every command to JSON; stderr carries logs and prompts
//! only. A command returns a `Serialize + Display` value rather than
//! printing inline. Each command's doc comment is its help text, which
//! makes `calendula <command> --help` the usage reference for both
//! humans and agents, and is why the README documents no per-command
//! usage.

mod account;
mod backend;
#[cfg(feature = "caldav")]
mod caldav;
mod cli;
mod config;
#[cfg(feature = "gcal")]
mod gcal;
#[cfg(feature = "pimdir")]
mod pimdir;
mod shared;
#[cfg(feature = "vdir")]
mod vdir;
#[cfg(any(feature = "caldav", feature = "vdir", feature = "pimdir"))]
mod wizard;

use std::{
    io::{IsTerminal, stdin},
    path::PathBuf,
};

use anyhow::Result;
use clap::{CommandFactory, Parser};
use pimalaya_cli::{
    error::ErrorReport,
    log::Logger,
    printer::{Printer, StdoutPrinter},
};
use pimalaya_config::toml::TomlConfig;

use crate::{cli::CalendulaCli, config::Config};

fn main() {
    let cli = CalendulaCli::parse();
    let mut printer = StdoutPrinter::new(&cli.json);
    let result = run(cli, &mut printer);
    ErrorReport::eval(&mut printer, result);
}

fn run(cli: CalendulaCli, printer: &mut StdoutPrinter) -> Result<()> {
    Logger::try_init(&cli.log)?;

    let config_paths = cli.config_paths.as_ref();
    let account = cli.account.name.as_deref();
    let backend = cli.backend;

    let Some(command) = cli.command else {
        return meet_bare_invocation(printer, config_paths, account.is_some());
    };

    command.execute(printer, config_paths, account, backend)
}

/// Meets a bare `calendula`, which is where a newcomer lands.
///
/// A missing configuration raises the offer, and everything else gets
/// the help: an existing configuration, a script, a JSON caller, and
/// `--account`, which names an account to act on and so reads as a
/// half-typed command rather than a first run. A file that exists but
/// fails to parse counts as a configuration, so the offer never proposes
/// to write over a broken one.
fn meet_bare_invocation(
    printer: &mut StdoutPrinter,
    config_paths: &[PathBuf],
    named_account: bool,
) -> Result<()> {
    let configured = Config::from_paths_or_default(config_paths)
        .ok()
        .flatten()
        .is_some();

    if !configured && !named_account && !printer.is_json() && stdin().is_terminal() {
        let path = Config::target_path(config_paths)?;

        // NOTE: nothing to run after the offer, so a declined one falls
        // back to the help; the wizard says what to run next when it ran.
        if cli::offer_configuration(printer, config_paths, &path)? {
            return Ok(());
        }
    }

    CalendulaCli::command().print_help()?;

    Ok(())
}
