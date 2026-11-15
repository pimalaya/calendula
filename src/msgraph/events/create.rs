//! # Event create command
//!
//! Posts a raw Graph event body to a calendar.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{
    client::{MsgraphClient, calendar_refusal},
    events::{input::EventJsonArg, render::MsgraphEventOutput},
};

/// Create an event from a raw Graph event JSON body.
///
/// JSON output: the raw Graph event the server created.
#[derive(Debug, Parser)]
pub struct MsgraphEventCreateCommand {
    /// Calendar id; omit for the default calendar.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR-ID")]
    pub calendar: Option<String>,
    /// The raw Graph event JSON body.
    #[command(flatten)]
    pub json: EventJsonArg,
}

impl MsgraphEventCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let event = self.json.read()?;
        let created = client
            .event_create(self.calendar.as_deref(), &event)
            .map_err(calendar_refusal)?
            .response;

        printer.out(MsgraphEventOutput(created))
    }
}
