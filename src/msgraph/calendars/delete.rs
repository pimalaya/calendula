//! # Calendar delete command
//!
//! Deletes a calendar and everything it holds.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    msgraph::client::{MsgraphClient, calendar_refusal},
    shared::receipt::IdReceipt,
};

/// Delete a calendar and every event it contains.
///
/// JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct MsgraphCalendarDeleteCommand {
    /// Calendar id.
    #[arg(value_name = "CALENDAR-ID")]
    pub calendar_id: String,
}

impl MsgraphCalendarDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        client
            .calendar_delete(&self.calendar_id)
            .map_err(calendar_refusal)?;

        printer.out(IdReceipt {
            id: self.calendar_id,
            done: "Calendar deleted",
        })
    }
}
