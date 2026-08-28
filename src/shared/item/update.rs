use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::{arg::CalendarIdArg, client::CalendarClient, ical::IcalArg};

/// Overwrite an existing iCalendar item from an iCalendar source.
///
/// Use `--if-match` to gate the write on a previously-read ETag when
/// the backend supports optimistic concurrency.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct ItemUpdateCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// Stable item identifier (iCal `UID`).
    #[arg(value_name = "ITEM-ID")]
    pub item_id: String,

    /// Optional `If-Match` precondition (ETag).
    #[arg(long, value_name = "ETAG")]
    pub if_match: Option<String>,

    #[command(flatten)]
    pub ical: IcalArg,
}

impl ItemUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        let contents = self.ical.read()?;

        client.update_item(
            &calendar_id,
            &self.item_id,
            contents,
            self.if_match.as_deref(),
        )?;
        printer.out(Message::new("Item successfully updated"))
    }
}
