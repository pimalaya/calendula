//! # Calendar get command
//!
//! Reads a single calendar by id.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{
    calendars::render::MsgraphCalendarOutput,
    client::{MsgraphClient, calendar_refusal},
};

/// GET a calendar by id.
///
/// JSON output: the raw Graph calendar object.
#[derive(Debug, Parser)]
pub struct MsgraphCalendarGetCommand {
    /// Calendar id.
    #[arg(value_name = "CALENDAR-ID")]
    pub calendar_id: String,
}

impl MsgraphCalendarGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let calendar = client
            .calendar_get(&self.calendar_id)
            .map_err(calendar_refusal)?
            .response;
        printer.out(MsgraphCalendarOutput(calendar))
    }
}
