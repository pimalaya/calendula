use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::{arg::CalendarIdArg, client::CalendarClient, ical::IcalArg};

/// Overwrite an existing event from an iCalendar source.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct EventUpdateCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// Stable event identifier (iCal `UID`).
    #[arg(value_name = "EVENT-ID")]
    pub event_id: String,

    /// Optional `If-Match` precondition (ETag).
    #[arg(long, value_name = "ETAG")]
    pub if_match: Option<String>,

    #[command(flatten)]
    pub ical: IcalArg,
}

impl EventUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        let contents = self.ical.read()?;

        client.update_item(
            &calendar_id,
            &self.event_id,
            contents,
            self.if_match.as_deref(),
        )?;
        printer.out(Message::new("Event successfully updated"))
    }
}
