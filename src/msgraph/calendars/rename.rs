//! # Calendar rename command
//!
//! Patches the name of a calendar.

use anyhow::Result;
use clap::Parser;
use io_msgraph::v1::{field::MsgraphField, rest::users::calendars::MsgraphCalendar};
use pimalaya_cli::printer::Printer;

use crate::msgraph::{
    calendars::render::MsgraphCalendarOutput,
    client::{MsgraphClient, calendar_refusal},
};

/// Rename a calendar (PATCH its `name`).
///
/// JSON output: the raw Graph calendar object after the update.
#[derive(Debug, Parser)]
pub struct MsgraphCalendarRenameCommand {
    /// Calendar id.
    #[arg(value_name = "CALENDAR-ID")]
    pub calendar_id: String,
    /// New name.
    #[arg(value_name = "NAME")]
    pub name: String,
}

impl MsgraphCalendarRenameCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let calendar = MsgraphCalendar {
            name: MsgraphField::Set(self.name),
            ..Default::default()
        };
        let updated = client
            .calendar_update(&self.calendar_id, &calendar)
            .map_err(calendar_refusal)?
            .response;

        printer.out(MsgraphCalendarOutput(updated))
    }
}
