//! # RMCOL command
//!
//! Runs a WebDAV `DELETE` against a whole calendar collection, the
//! counterpart of `caldav mkcalendar`.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{caldav::client::CaldavClient, shared::receipt::IdReceipt};

/// DELETE a calendar collection and every item it contains.
///
/// JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct CaldavRmcolCommand {
    /// Identifier of the calendar (its last URL segment).
    #[arg(value_name = "CALENDAR")]
    pub calendar_id: String,
}

impl CaldavRmcolCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CaldavClient) -> Result<()> {
        client.delete_calendar(&self.calendar_id)?;

        printer.out(IdReceipt {
            id: self.calendar_id,
            done: "Calendar deleted",
        })
    }
}
