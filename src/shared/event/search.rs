//! # Event search
//!
//! The `calendula event search` command, listing a page of the account's
//! events a pimdir SEARCH query matches, answered by a pimdir store's
//! index.

use std::fmt;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use io_pimdir::search::{plan::PimdirSearchCursor, query::PimdirQuery};
use jiff::tz::TimeZone;
use pimalaya_cli::{
    clap::args::PageTokenArg,
    listing::{Listing, ListingPage},
    printer::Printer,
    sanitize,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    capability::Capability,
    error::{ItemError, partial_when},
    pimdir::{
        backend::PimdirBackend,
        index::parse_query,
        search::{PimdirSearchSortArg, page_limit, search_page},
    },
    shared::{
        client::CalendarClient,
        event::{
            Event,
            list::{EVENT_NOUNS, local_zone, row_times},
        },
    },
};

/// Search the account's events with the pimdir query language.
///
/// The query is pimdir SEARCH.md §8, the one `pimdir search` and
/// cardamum's `card search` take: words and "phrases" (a trailing `*` for
/// a prefix), `field:value` terms (`summary:`, `location:`, `attendee:`,
/// `organizer:`, `category:`, `date:2026-11-09`, `date:2026-01..2026-03`
/// and the others), `and`, `or`, `not` or a leading `-` (its word after
/// `--`), and parentheses. `date:` and `when:` also take an instant
/// (`when:2026-11-23T10:00`) and amounts relative to now in `min`, `h`,
/// `d`, `w`, `mo` and `y` (`date:2h..`). `label:work` and
/// `property:<key>=<value>` read the labels and properties the store keeps
/// on an item. A word or a value beginning with `~`, one of two characters
/// or more beginning with `/`, and a value beginning with `{` are reserved
/// (`reserved-form`): quote them to search them as text. It is parsed
/// before any backend is reached, so a query is refused the same way
/// everywhere (`query-refused`). It reads every calendar of the account,
/// or the one `-k` names.
///
/// A pimdir store's index answers it, as `pimdir search` does; every
/// other backend refuses it (`capability-missing`, `event.search`). An
/// event is looked up by its iCalendar UID with `event read --uid`.
///
/// A page ends with the token of the place it stopped, which
/// `--page-token` takes back; the last page has none.
///
/// JSON output: `{"events": [{...event list row}], "nextPage",
/// "complete", "errors": [{"id", "code", "error", "sources"}]}`:
/// `complete` is `false` when the index could not see every event of the
/// scope (`pimdir search` says what it missed), and `errors` lists the
/// events the store could not read, the command then exiting 3.
#[derive(Debug, Parser)]
pub struct EventSearchCommand {
    /// Calendar to search, every one of the account's when omitted.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR-ID")]
    pub calendar: Option<String>,
    /// 1-indexed page number to fetch.
    #[arg(short, long, value_name = "N")]
    pub page: Option<u32>,
    /// Number of events per page, the `event list` page size by default,
    /// zero listing every one.
    #[arg(short = 's', long, value_name = "N")]
    pub page_size: Option<u32>,
    #[command(flatten)]
    pub page_token: PageTokenArg,
    /// The order of the events.
    #[arg(long, value_enum, default_value_t)]
    pub sort: EventSearchSort,
    /// The query, its words joined by spaces.
    ///
    /// A word starting with `-`, a negated term, goes after `--`, the
    /// options before it: `event search -s 5 -- budget -lisbon`.
    #[arg(value_name = "QUERY", required = true)]
    pub query: Vec<String>,
}

impl EventSearchCommand {
    /// Searches the events in scope and prints one page of them.
    pub fn execute(self, printer: &mut impl Printer, client: &mut CalendarClient) -> Result<()> {
        let words = self.query.join(" ");
        let query = parse_query(&self.query, printer.is_json())?;

        client.require(&[Capability::EventSearch])?;

        let search = EventSearch {
            query,
            calendar: self.calendar.clone(),
            sort: self.sort,
            page_size: self.page_size,
            page: self.page,
            token: self.page_token.token,
        };
        let found = client.search_events(&search)?;

        let mut listing = Listing::new(&client.name, Some(client.backend_name()));
        listing.head("Calendar", self.calendar.as_deref().unwrap_or("all"));
        listing.head("Query", &words);
        let page = match found.page {
            Some(page) => ListingPage::Numbered {
                page: page.into(),
                size: found.size.into(),
            },
            None => ListingPage::Token,
        };
        let next = found.next_page.as_deref();
        listing.page(page, found.events.len(), EVENT_NOUNS, next);
        listing.next_page = next.map(|next| format!("--page-token {}", sanitize::text(next)));

        if !found.complete {
            listing.head(
                "Incomplete",
                "the index could not see every event; `pimdir search` says what it missed",
            );
        }

        let output = EventSearchOutput {
            listing,
            zone: local_zone(),
            events: found.events,
            next_page: found.next_page,
            complete: found.complete,
            errors: found.errors,
        };

        printer.out(&output)?;
        partial_when(&output.errors)
    }
}

/// The order of an event search (pimdir SEARCH §8.2).
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum EventSearchSort {
    /// Latest first, a series by its first occurrence.
    #[default]
    #[value(alias = "date")]
    NewestFirst,
    /// Earliest first, a series by its first occurrence.
    OldestFirst,
    /// By summary, A to Z.
    #[value(alias = "name")]
    Title,
    /// Best match of the query's words first.
    Relevance,
}

/// A search as the command asks it of a backend.
pub struct EventSearch {
    /// The parsed query.
    pub query: PimdirQuery,
    /// The calendar searched, every one of the account's when `None`.
    pub calendar: Option<String>,
    /// The order of the events.
    pub sort: EventSearchSort,
    /// The size of a page, the `event list` one when `None`.
    pub page_size: Option<u32>,
    /// The page asked by number, from 1, when no token is given.
    pub page: Option<u32>,
    /// The `nextPage` a previous page printed.
    pub token: Option<String>,
}

/// One page of events, as a backend answers it.
#[derive(Debug, Default)]
pub struct EventSearchPage {
    /// The events of the page.
    pub events: Vec<Event>,
    /// The page listed by number, `None` for one a token reached.
    pub page: Option<u32>,
    /// The size of a page.
    pub size: u32,
    /// The token of the next page, `None` on the last one.
    pub next_page: Option<String>,
    /// Whether the answer saw every event of the scope.
    pub complete: bool,
    /// The error of each event that could not be read.
    pub errors: Vec<ItemError>,
}

/// The `event search` output.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventSearchOutput {
    /// The header the events print under, its page placed.
    #[serde(skip)]
    pub listing: Listing<'static>,
    /// The zone a timed event's start and end print in.
    #[serde(skip)]
    pub zone: TimeZone,
    /// The events of the page, each its `event list` row.
    pub events: Vec<Event>,
    /// The token of the next page, `null` once the answer is complete,
    /// given back through `--page-token`.
    pub next_page: Option<String>,
    /// Whether the answer saw every event of the scope: `false` when the
    /// pimdir index lags the store or misses a body.
    pub complete: bool,
    /// The error of each event that could not be read, in the order met.
    pub errors: Vec<ItemError>,
}

impl fmt::Display for EventSearchOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut listing = self.listing.clone();

        for event in &self.events {
            let (start, end) = row_times(event, &self.zone);

            listing.items.push(vec![
                sanitize::text(&event.id).into_owned(),
                sanitize::text(&start).into_owned(),
                sanitize::text(&end).into_owned(),
                sanitize::text(&event.calendar_id).into_owned(),
                sanitize::text(event.summary.as_deref().unwrap_or_default()).into_owned(),
            ]);
        }

        for error in &self.errors {
            listing.errors.push(error);
        }

        write!(f, "{listing}")
    }
}

/// The page of `search` the pimdir store's index answers, as `pimdir
/// search` pages it: its token is the index's cursor, and `complete` its
/// coverage verdict.
pub fn pimdir_page(backend: &PimdirBackend, search: &EventSearch) -> Result<EventSearchPage> {
    let sort = match search.sort {
        EventSearchSort::NewestFirst => PimdirSearchSortArg::NewestFirst,
        EventSearchSort::OldestFirst => PimdirSearchSortArg::OldestFirst,
        EventSearchSort::Title => PimdirSearchSortArg::Title,
        EventSearchSort::Relevance => PimdirSearchSortArg::Relevance,
    };
    let size = page_limit(backend, search.page_size);
    let (found, page) = search_page(
        backend,
        search.query.clone(),
        search.calendar.as_deref(),
        sort,
        size,
        search.page,
        search.token.as_deref(),
    )?;

    Ok(EventSearchPage {
        events: found.hits.into_iter().map(|hit| hit.event).collect(),
        page,
        size,
        next_page: found.next.as_ref().map(PimdirSearchCursor::token),
        complete: found.coverage.complete,
        errors: found.errors,
    })
}
