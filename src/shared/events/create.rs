use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::{arg::CalendarIdArg, client::CalendarClient, ical::IcalArg};

/// Create a new event from an iCalendar source.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct EventCreateCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    #[command(flatten)]
    pub ical: IcalArg,
}

impl EventCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        let contents = self.ical.read()?;

        let id = client.create_item(&calendar_id, contents)?;
        printer.out(Message::new(format!("Event `{id}` successfully created")))
    }
}
