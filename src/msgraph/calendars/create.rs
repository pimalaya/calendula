//! # Calendar create command
//!
//! Creates a calendar from its name.

use anyhow::Result;
use clap::Parser;
use io_msgraph::v1::{field::MsgraphField, rest::users::calendars::MsgraphCalendar};
use pimalaya_cli::printer::Printer;

use crate::msgraph::{calendars::render::MsgraphCalendarOutput, client::MsgraphClient};

/// Create a calendar.
///
/// JSON output: the raw Graph calendar object the server created.
#[derive(Debug, Parser)]
pub struct MsgraphCalendarCreateCommand {
    /// Name of the calendar to create.
    #[arg(value_name = "NAME")]
    pub name: String,
}

impl MsgraphCalendarCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let calendar = MsgraphCalendar {
            name: MsgraphField::Set(self.name),
            ..Default::default()
        };
        let created = client.calendar_create(&calendar)?.response;

        printer.out(MsgraphCalendarOutput(created))
    }
}
