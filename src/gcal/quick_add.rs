use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::gcal::client::GcalClient;

/// Create an event from a sentence.
///
/// The text is parsed server-side, so "Lunch with Ada tomorrow at
/// noon" becomes a dated event without a hand-written iCalendar. What
/// Google understands is its own business, which is why this cannot be
/// a shared command.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct GcalQuickAddCommand {
    /// The calendar the event lands in. Falls back to
    /// `calendar.default`.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR-ID")]
    pub calendar_id: Option<String>,

    /// The sentence describing the event.
    #[arg(value_name = "TEXT")]
    pub text: String,
}

impl GcalQuickAddCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GcalClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar_id)?;

        let created = client
            .event_quick_add(&calendar_id, &self.text, None)?
            .response;
        let id = created.id.unwrap_or_default();

        printer.out(Message::new(format!("Event `{id}` successfully created")))
    }
}
