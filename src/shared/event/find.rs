//! # Event find
//!
//! The `calendula event find` command, looking an event up by the
//! iCalendar `UID` it carries rather than by the id its backend gave it.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    capability::Capability,
    shared::{
        arg::CalendarIdArg,
        client::CalendarClient,
        event::{
            Event,
            list::{EVENT_NOUNS, EventListOutput, local_zone},
        },
        ical::IcalFamily,
        item::CalendarItemQuery,
        listing::Listing,
    },
};

/// Find the events carrying an iCalendar UID.
///
/// An invitation names its event by `UID`, which is not the id a backend
/// addresses it by: this is the bridge. Every VEVENT carrying the UID is
/// listed unexpanded, a series once and each of its overrides once, with
/// the id the other `event` commands take.
///
/// The calendar is scanned whole, so this costs a full listing on a
/// remote backend and a full read on a local one. On pimdir, whose stored
/// summary names each resource's `UID`, only the bodies that may carry
/// this one are read.
///
/// JSON output: `{"events": [...], "nextPage": null}`, shaped as `event
/// list` prints them.
#[derive(Debug, Parser)]
pub struct EventFindCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// The iCalendar UID to look for, compared exactly.
    #[arg(value_name = "UID")]
    pub uid: String,
}

impl EventFindCommand {
    pub fn execute(self, printer: &mut impl Printer, client: &mut CalendarClient) -> Result<()> {
        client.require(&[Capability::ItemList])?;

        let calendar_id = client.account.calendar_id(self.calendar.id)?;

        let query = CalendarItemQuery {
            kind: IcalFamily::Event.kind(),
            uid: Some(&self.uid),
            ..Default::default()
        };
        let listed = client.list_items(&calendar_id, None, None, query)?;

        let events: Vec<Event> = listed
            .items
            .iter()
            .flat_map(Event::project)
            .filter(|event| event.uid == self.uid)
            .collect();

        let mut ids: Vec<&str> = events.iter().map(|event| event.id.as_str()).collect();
        ids.dedup();

        let mut listing = Listing::new(&client.name, client.backend_name(), EVENT_NOUNS);
        listing.push("Calendar", &calendar_id);
        listing.push("UID", &self.uid);
        listing.count = ids.len();

        printer.out(EventListOutput {
            listing,
            zone: local_zone(),
            events,
            next_page: None,
        })
    }
}
