use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::{arg::CalendarIdArg, client::CalendarClient, ical::IcalArg};

/// Create a new journal entry from an iCalendar source.
///
/// The source is stored as given: a backend that cannot model a
/// VJOURNAL refuses it by name rather than emulating one.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct JournalCreateCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    #[command(flatten)]
    pub ical: IcalArg,
}

impl JournalCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        let contents = self.ical.read()?;

        let id = client.create_item(&calendar_id, contents)?;
        printer.out(Message::new(format!(
            "Journal entry `{id}` successfully created"
        )))
    }
}
