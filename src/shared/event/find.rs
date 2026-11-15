//! # Event search
//!
//! The `calendula event search` command, looking an event up by the
//! iCalendar `UID` it carries rather than by the id its backend gave it.

use anyhow::{Context, Result};
use clap::Parser;
use pimalaya_cli::{clap::args::PageTokenArg, listing::Listing, printer::Printer};

use crate::{
    capability::Capability,
    error,
    shared::{
        arg::CalendarIdArg,
        client::CalendarClient,
        event::{
            Event,
            list::{EVENT_NOUNS, EventListOutput, local_zone},
        },
        ical::IcalFamily,
        item::{CalendarItemQuery, list::listing_page},
    },
};

/// Search the events carrying an iCalendar UID.
///
/// An invitation names its event by `UID`, which is not the id a backend
/// addresses it by: this is the bridge, the other `event` commands taking
/// the id alone. Every VEVENT carrying the UID is listed unexpanded, a
/// series once and each of its overrides once, with the id the other
/// `event` commands take.
///
/// The calendar is scanned whole, so this costs a full listing on a
/// remote backend and a full read on a local one. On pimdir, whose stored
/// summary names each resource's `UID`, only the bodies that may carry
/// this one are read.
///
/// Every match is listed on one page unless `--page-size` cuts it, a page
/// counting resources as `event list` does: a series and its overrides
/// are one.
///
/// An item whose iCalendar does not parse, which may carry the UID, is
/// named by an `Error:` line at the end, and the command exits 3.
///
/// JSON output: `{"events": [...], "nextPage", "errors": [...]}`, shaped
/// as `event list` prints them, `nextPage` the number of the page
/// following this one as a string, which `--page-token` takes back, or
/// `null` once the matches are complete.
#[derive(Debug, Parser)]
pub struct EventFindCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// 1-indexed page number. Defaults to 1.
    #[arg(short, long, value_name = "N")]
    pub page: Option<u32>,

    /// Number of matching items per page, zero or omitted listing every
    /// one.
    #[arg(short = 's', long, value_name = "N")]
    pub page_size: Option<u32>,

    #[command(flatten)]
    pub page_token: PageTokenArg,

    /// The iCalendar UID to look for, compared exactly.
    #[arg(value_name = "UID")]
    pub uid: String,
}

impl EventFindCommand {
    pub fn execute(self, printer: &mut impl Printer, client: &mut CalendarClient) -> Result<()> {
        client.require(&[Capability::ItemList])?;

        let page = match &self.page_token.token {
            Some(token) => Some(
                token
                    .parse::<u32>()
                    .with_context(|| format!("Invalid page token `{token}`"))?,
            ),
            None => self.page,
        };

        let page_size = self.page_size.filter(|size| *size > 0);
        let page = page_size.map(|_| page.unwrap_or(1).max(1));

        let calendar_id = client.account.calendar_id(self.calendar.id)?;

        let query = CalendarItemQuery {
            kind: IcalFamily::Event.kind(),
            uid: Some(&self.uid),
            ..Default::default()
        };
        let listed = client.list_items(&calendar_id, None, None, query)?;

        let mut errors = listed.errors;
        let (mut events, _) = Event::read_all(&listed.items, &mut errors, Event::try_project);
        events.retain(|event| event.uid.as_deref() == Some(self.uid.as_str()));

        // NOTE: a page counts resources, the events of one item following
        // each other.
        let mut ids: Vec<String> = events.iter().map(|event| event.id.clone()).collect();
        ids.dedup();

        let (held, next_page) = match (page, page_size) {
            (Some(page), Some(size)) => {
                let skip = (page as usize - 1) * size as usize;
                let window: Vec<String> = ids.into_iter().skip(skip).collect();
                let more = window.len() > size as usize;
                let window = &window[..window.len().min(size as usize)];
                events.retain(|event| window.contains(&event.id));
                (window.len(), more.then(|| (page + 1).to_string()))
            }
            _ => (ids.len(), None),
        };

        let mut listing = Listing::new(&client.name, Some(client.backend_name()));
        listing.head("Calendar", &calendar_id);
        listing.head("UID", &self.uid);
        let shown = listing_page(page, page_size);
        listing.page(shown, held, EVENT_NOUNS, next_page.as_deref());

        let output = EventListOutput {
            listing,
            zone: local_zone(),
            events,
            next_page,
            errors,
        };

        printer.out(&output)?;
        error::partial_when(&output.errors)
    }
}
