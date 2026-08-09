use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::{arg::CalendarIdArg, client::CalendarClient, ical::IcalArg};

/// Overwrite an existing journal entry from an iCalendar source.
///
/// Use `--if-match` to gate the write on a previously-read ETag when
/// the backend supports optimistic concurrency.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct JournalUpdateCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// Stable journal entry identifier.
    #[arg(value_name = "JOURNAL-ID")]
    pub journal_id: String,

    /// Optional `If-Match` precondition (ETag).
    #[arg(long, value_name = "ETAG")]
    pub if_match: Option<String>,

    #[command(flatten)]
    pub ical: IcalArg,
}

impl JournalUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        let contents = self.ical.read()?;

        client.update_item(
            &calendar_id,
            &self.journal_id,
            contents,
            self.if_match.as_deref(),
        )?;
        printer.out(Message::new("Journal entry successfully updated"))
    }
}
