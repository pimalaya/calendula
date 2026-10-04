//! # Event find
//!
//! The `calendula event find` command, looking an event up by the
//! iCalendar `UID` it carries rather than by the id its backend gave it.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::shared::{
    arg::CalendarIdArg,
    client::CalendarClient,
    event::{Event, list::EventListOutput},
    ical::IcalFamily,
    item::CalendarItemQuery,
};

/// Find the events carrying an iCalendar UID.
///
/// An invitation names its event by `UID`, which is not the id a backend
/// addresses it by: this is the bridge. Every VEVENT carrying the UID is
/// listed unexpanded, a series once and each of its overrides once, with
/// the id the other `event` commands take.
///
/// The calendar is scanned whole, so this costs a full listing on a
/// remote backend and a full read on a local one.
///
/// JSON output: `{"events": [...]}`, shaped as `event list` prints them.
#[derive(Debug, Parser)]
pub struct EventFindCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// The iCalendar UID to look for, compared exactly.
    #[arg(value_name = "UID")]
    pub uid: String,

    /// Maximum width of the rendered table, in terminal columns.
    #[arg(long = "max-width", short = 'w', value_name = "COLUMNS")]
    pub max_width: Option<u16>,
}

impl EventFindCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;

        let items = client.list_items(
            &calendar_id,
            CalendarItemQuery {
                kind: IcalFamily::Event.kind(),
                ..Default::default()
            },
        )?;

        let events = items
            .iter()
            .flat_map(Event::project)
            .filter(|event| event.uid == self.uid)
            .collect();

        printer.out(EventListOutput::new(&client, self.max_width, events))
    }
}
