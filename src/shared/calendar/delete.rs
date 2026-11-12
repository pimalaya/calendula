//! # Calendar delete
//!
//! The `calendar delete` command, removing a calendar and everything in
//! it.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    capability::Capability,
    shared::{
        client::CalendarClient,
        receipt::{Write, WriteReceipt},
    },
};

/// Delete the given calendar and every item it contains.
///
/// JSON output: `{"id", "queued"}`.
#[derive(Debug, Parser)]
pub struct CalendarDeleteCommand {
    /// Calendar to delete.
    ///
    /// Mandatory: unlike the other shared-API commands, deletion never falls
    /// back to the `calendar.default` config.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR-ID")]
    pub id: String,
}

impl CalendarDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        client.require(&[Capability::CalendarDelete])?;
        client.delete_calendar(&self.id)?;

        printer.out(WriteReceipt {
            id: self.id,
            queued: client.queues_writes(),
            what: "Calendar",
            write: Write::Delete,
        })
    }
}
