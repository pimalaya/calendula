//! # CalDAV calendar delete
//!
//! The `calendula caldav delete` command, removing one calendar
//! collection from the server.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{caldav::client::CaldavClient, shared::receipt::IdReceipt};

/// Delete a CalDAV calendar by id.
///
/// The DELETE takes the collection and every resource filed in it.
///
/// JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct CaldavCalendarDeleteCommand {
    /// Calendar identifier (last path segment of the calendar URL).
    #[arg(value_name = "ID")]
    pub id: String,
}

impl CaldavCalendarDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CaldavClient) -> Result<()> {
        client.delete_calendar(&self.id)?;

        printer.out(IdReceipt {
            id: self.id,
            done: "Calendar deleted",
        })
    }
}
