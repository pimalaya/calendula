//! # REPORT command
//!
//! Dispatches the typed CalDAV REPORTs and the raw escape hatch.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::caldav::{
    client::{CaldavClient, check_calendar_id, check_item_id},
    report::{
        multiget::CaldavReportMultigetCommand, query::CaldavReportQueryCommand,
        raw::CaldavReportRawCommand, sync::CaldavReportSyncCommand,
    },
};

/// Run a CalDAV REPORT (RFC 4791 §7, RFC 6578).
///
/// The typed subcommands cover the three standard reports; `raw` sends
/// an arbitrary XML body for anything else.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum CaldavReportCommand {
    Query(CaldavReportQueryCommand),
    Multiget(CaldavReportMultigetCommand),
    Sync(CaldavReportSyncCommand),
    Raw(CaldavReportRawCommand),
}

impl CaldavReportCommand {
    /// Refuses, before anything connects, a calendar or item id no CalDAV
    /// resource can have (see [`CaldavCommand::check`]).
    ///
    /// [`CaldavCommand::check`]: crate::caldav::cli::CaldavCommand::check
    pub fn check(&self) -> Result<()> {
        match self {
            Self::Query(cmd) => check_calendar_id(&cmd.calendar_id),
            Self::Multiget(cmd) => {
                check_calendar_id(&cmd.calendar_id)?;
                cmd.item_ids.iter().try_for_each(|id| check_item_id(id))
            }
            Self::Sync(cmd) => check_calendar_id(&cmd.calendar_id),
            Self::Raw(cmd) => check_calendar_id(&cmd.calendar_id),
        }
    }

    pub fn execute(self, printer: &mut impl Printer, client: CaldavClient) -> Result<()> {
        match self {
            Self::Query(cmd) => cmd.execute(printer, client),
            Self::Multiget(cmd) => cmd.execute(printer, client),
            Self::Sync(cmd) => cmd.execute(printer, client),
            Self::Raw(cmd) => cmd.execute(printer, client),
        }
    }
}
