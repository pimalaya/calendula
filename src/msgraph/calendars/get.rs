//! # Calendar get command
//!
//! Reads a single calendar by id.

use anyhow::Result;
use clap::Parser;
use io_msgraph::v1::rest::users::calendars::MsgraphCalendar;
use pimalaya_cli::printer::Printer;

use crate::{
    msgraph::{
        calendars::render::MsgraphCalendarOutput,
        client::{MsgraphClient, calendar_refusal, coded},
    },
    shared::raw_json::RawRead,
};

/// GET a calendar by id.
///
/// JSON output: the Graph calendar object as Graph sent it.
#[derive(Debug, Parser)]
pub struct MsgraphCalendarGetCommand {
    /// Calendar id.
    #[arg(value_name = "CALENDAR-ID")]
    pub calendar_id: String,
}

impl MsgraphCalendarGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let value = client
            .raw_get(&["calendars", &self.calendar_id])
            .map_err(|err| coded(err, calendar_refusal))?;
        let text = serde_json::from_value::<MsgraphCalendar>(value.clone())
            .map(|calendar| MsgraphCalendarOutput(calendar).to_string())
            .unwrap_or_default();

        printer.out(RawRead { value, text })
    }
}
