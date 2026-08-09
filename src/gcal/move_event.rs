use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::gcal::client::GcalClient;

/// Move an event to another calendar.
///
/// The shared API would have to emulate this as a create plus a
/// delete, which mints a new id and drops whatever the projection does
/// not model. Google relocates the event itself, so it keeps both.
///
/// Only a single event moves: an instance of a recurring series cannot
/// leave its series.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct GcalMoveCommand {
    /// The calendar currently holding the event. Falls back to
    /// `calendar.default`.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR-ID")]
    pub calendar_id: Option<String>,

    /// The event to move.
    #[arg(value_name = "EVENT-ID")]
    pub event_id: String,

    /// The calendar the event moves to.
    #[arg(value_name = "DESTINATION-ID")]
    pub destination: String,
}

impl GcalMoveCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GcalClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar_id)?;

        client.event_move(&calendar_id, &self.event_id, &self.destination, None)?;

        printer.out(Message::new(format!(
            "Event `{}` successfully moved to `{}`",
            self.event_id, self.destination
        )))
    }
}
