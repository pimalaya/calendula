use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::{arg::CalendarIdArg, client::CalendarClient, ical::IcalArg};

/// Overwrite an existing todo from an iCalendar source.
///
/// The whole component is replaced, so marking a task done means
/// supplying an iCalendar carrying the new STATUS and
/// PERCENT-COMPLETE. Use `--if-match` to gate the write on a
/// previously-read ETag when the backend supports optimistic
/// concurrency.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct TodoUpdateCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// Stable todo identifier.
    #[arg(value_name = "TODO-ID")]
    pub todo_id: String,

    /// Optional `If-Match` precondition (ETag).
    #[arg(long, value_name = "ETAG")]
    pub if_match: Option<String>,

    #[command(flatten)]
    pub ical: IcalArg,
}

impl TodoUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        let contents = self.ical.read()?;

        client.update_item(
            &calendar_id,
            &self.todo_id,
            contents,
            self.if_match.as_deref(),
        )?;
        printer.out(Message::new("Todo successfully updated"))
    }
}
