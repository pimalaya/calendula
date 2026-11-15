//! # Pimdir search
//!
//! The `pimdir search` command.

use std::fmt;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use io_pimdir::client::search::{PimdirSearchOptions, PimdirSearchSort};
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
    error::{ItemError, partial_when},
    pimdir::{
        backend::PimdirBackend,
        index::{PimdirCalendarCoverage, PimdirCalendarHit, parse_query, search_clock},
    },
    shared::event::list::{EVENT_NOUNS, local_zone, row_times},
};

/// Search the account's events with the pimdir query language.
///
/// The query is pimdir SEARCH.md §8: words, "phrases", `field:value`
/// terms (`summary:`, `body:`, `location:`, `attendee:`, `organizer:`,
/// `category:`, `date:2026-11-09`, `date:2026-01..2026-03`), `and`, `or`,
/// `not` or `-`, and parentheses, a query holding a `-` term written after
/// `--`. Every calendar is searched unless `-k` names one.
///
/// `date:` matches occurrences, a series included. Days, `today` and
/// `yesterday` are read in the local zone (`TZ`, then the system's).
///
/// The answer is the index as the sync engine last refreshed it: a queued
/// write is not in it. Each hit prints on one line, as `event list` prints
/// an event, its calendar and what a text term matched before its summary;
/// an `Incomplete:` line above them says what the answer could not see, a
/// `Zones:` line the index's zone when a `date:` term reads its days in
/// another than the index read floating times in.
///
/// A page names the number of the next one, which `--page-token` takes;
/// the last page names none. An event the store cannot read is an item
/// error, printed after the others, and the command exits 3.
///
/// JSON output: `{"hits": [{...the event projection, "calendarId",
/// "matched", "snippet"}], "nextPage", "coverage": {"complete",
/// "bodiless", "missing", "unparseable", "unindexed", "behind", "since",
/// "until", "filling", "horizonLeft", "cut", "zoneDiffers"}, "errors"}`.
#[derive(Debug, Parser)]
pub struct PimdirSearchCommand {
    /// Calendar to search alone; every calendar when omitted.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR-ID")]
    pub calendar: Option<String>,

    /// Order of the hits.
    #[arg(long, value_enum, default_value_t)]
    pub sort: PimdirSearchSortArg,

    /// 1-indexed page number. Defaults to 1.
    #[arg(short, long, value_name = "N")]
    pub page: Option<u32>,

    /// Number of hits per page, the `event list` page size by default.
    #[arg(short = 's', long, value_name = "N")]
    pub page_size: Option<u32>,

    #[command(flatten)]
    pub page_token: PageTokenArg,

    /// The query, its words joined by spaces.
    ///
    /// An option is read wherever it stands, before the query or after
    /// it. A word starting with `-`, a negated term, goes after `--`, the
    /// options before it: `pimdir search -s 5 -- budget -lisbon`.
    #[arg(value_name = "QUERY", required = true)]
    pub query: Vec<String>,
}

impl PimdirSearchCommand {
    pub fn execute(self, printer: &mut impl Printer, backend: PimdirBackend) -> Result<()> {
        let words = self.query.join(" ");
        let query = parse_query(&self.query, printer.is_json())?;
        if let Some(calendar) = &self.calendar {
            backend.known_collection(calendar)?;
        }

        let page = match &self.page_token.token {
            Some(token) => token
                .parse::<u32>()
                .ok()
                .filter(|page| *page > 0)
                .with_context(|| format!("Invalid page token `{token}`"))?,
            None => self.page.unwrap_or(1).max(1),
        };

        let account = &backend.account;
        let page_size = self.page_size.unwrap_or(account.events_list_page_size());
        let limit = page_size.max(1);
        let options = PimdirSearchOptions {
            sort: self.sort.into(),
            limit: limit as usize,
            offset: (page as usize - 1).saturating_mul(limit as usize),
            clock: search_clock(),
        };

        let answer = backend
            .client
            .search_items(query, self.calendar.as_deref(), &options)?;

        let next_page = answer.more.then(|| (page + 1).to_string());

        let mut listing = Listing::new(&backend.name, Some("pimdir"));
        listing.head("Calendar", self.calendar.as_deref().unwrap_or("all"));
        listing.head("Query", &words);
        let held = ListingPage::Numbered {
            page: page.into(),
            size: limit.into(),
        };
        listing.page(held, answer.hits.len(), EVENT_NOUNS, next_page.as_deref());
        answer.coverage.note(&mut listing);
        if let Some(zones) = &answer.zones {
            zones.note(&mut listing);
        }

        let output = PimdirSearchOutput {
            listing,
            zone: local_zone(),
            hits: answer.hits,
            next_page,
            coverage: answer.coverage,
            errors: answer.errors,
        };

        printer.out(&output)?;
        partial_when(&output.errors)
    }
}

/// The order of a search (pimdir SEARCH §8.2).
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum PimdirSearchSortArg {
    /// Latest first, a series by its first occurrence.
    #[default]
    Date,
    /// By summary, A to Z.
    Name,
    /// Best match first.
    Relevance,
}

impl From<PimdirSearchSortArg> for PimdirSearchSort {
    fn from(sort: PimdirSearchSortArg) -> Self {
        match sort {
            PimdirSearchSortArg::Date => Self::Date,
            PimdirSearchSortArg::Name => Self::Name,
            PimdirSearchSortArg::Relevance => Self::Relevance,
        }
    }
}

/// The `pimdir search` output.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirSearchOutput {
    /// The header the hits print under, its page placed.
    #[serde(skip)]
    pub listing: Listing<'static>,
    /// The zone a timed hit's start and end print in.
    #[serde(skip)]
    pub zone: TimeZone,
    /// The hits of this page.
    pub hits: Vec<PimdirCalendarHit>,
    /// The number of the page following this one, given back through
    /// `--page-token`, `null` once the answer is complete.
    pub next_page: Option<String>,
    /// What the answer could not see.
    pub coverage: PimdirCalendarCoverage,
    /// The error of each event the store could not read, in the order
    /// met.
    pub errors: Vec<ItemError>,
}

impl fmt::Display for PimdirSearchOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut listing = self.listing.clone();
        let matched = self.hits.iter().any(|hit| hit.matched.is_some());

        for hit in &self.hits {
            let (start, end) = row_times(&hit.event, &self.zone);

            let mut line = vec![
                sanitize::text(&hit.event.id).into_owned(),
                sanitize::text(&start).into_owned(),
                sanitize::text(&end).into_owned(),
                sanitize::text(&hit.calendar_id).into_owned(),
            ];

            if matched {
                let matched = hit.matched.map(|matched| matched.to_string());
                line.push(matched.unwrap_or_default());
            }

            line.push(sanitize::text(&hit.event.summary).into_owned());
            listing.items.push(line);
        }

        for error in &self.errors {
            listing.errors.push(error);
        }

        write!(f, "{listing}")
    }
}

#[cfg(test)]
mod tests {
    use jiff::tz::TimeZone;
    use pimalaya_cli::listing::{Listing, ListingPage};

    use crate::{
        error::{ErrorCode, ItemError},
        pimdir::{
            index::{
                PimdirCalendarCoverage, PimdirCalendarHit, PimdirCalendarMatched,
                PimdirCalendarZones,
            },
            search::PimdirSearchOutput,
        },
        shared::event::{Event, list::EVENT_NOUNS},
    };

    /// One hit on the first page of an incomplete answer naming its next
    /// page, and an event the store could not read.
    fn output() -> PimdirSearchOutput {
        let coverage = PimdirCalendarCoverage {
            bodiless: 1,
            counts_bodiless: true,
            ..Default::default()
        };
        let zones = PimdirCalendarZones {
            index: "UTC".into(),
            search: "Europe/Paris".into(),
        };

        let mut listing = Listing::new("perso", Some("pimdir"));
        listing.head("Calendar", "all");
        listing.head("Query", "budget");
        let page = ListingPage::Numbered { page: 1, size: 1 };
        listing.page(page, 1, EVENT_NOUNS, Some("2"));
        coverage.note(&mut listing);
        zones.note(&mut listing);

        PimdirSearchOutput {
            listing,
            zone: TimeZone::UTC,
            hits: vec![PimdirCalendarHit {
                event: Event {
                    id: "7".into(),
                    summary: "Budget review".into(),
                    start: "20261110".into(),
                    end: "20261111".into(),
                    all_day: true,
                    starts_at: Some("2026-11-10".into()),
                    ends_at: Some("2026-11-11".into()),
                    ..Default::default()
                },
                calendar_id: "cal".into(),
                matched: Some(PimdirCalendarMatched::Summary),
                snippet: Some("the\nbudget".into()),
            }],
            next_page: Some("2".into()),
            coverage,
            errors: vec![ItemError {
                id: Some("9".into()),
                code: Some(ErrorCode::ItemUnreadable),
                error: "Read item `9`".into(),
                sources: vec!["Invalid column type".into()],
            }],
        }
    }

    /// A hit prints on one line under the account, the calendar, the
    /// query, the page and what the answer could not see; an event the
    /// store could not read prints after the hits.
    #[test]
    fn a_hit_prints_on_one_line_and_an_error_last() {
        let expected = "\
Account:     perso (pimdir)
Calendar:    all
Query:       budget
Page:        1, event 1
Next page:   --page 2
Incomplete:  1 item(s) have no local body and matched on their summary alone
Zones:       the index reads floating times and dates in UTC, this search reads days in Europe/Paris

7  2026-11-10  2026-11-10  cal  summary  Budget review

Error:  9  item-unreadable  Read item `9`
";
        assert_eq!(output().to_string(), expected);
    }

    /// The page names the next one, which `--page-token` takes, and its
    /// errors, as every listing of the family: no `more`.
    #[test]
    fn a_page_names_the_next_and_its_errors() {
        let json = serde_json::to_value(output()).unwrap();
        let mut keys: Vec<&str> = json.as_object().unwrap().keys().map(|k| &**k).collect();
        keys.sort();
        assert_eq!(keys, ["coverage", "errors", "hits", "nextPage"]);
        assert_eq!(json["nextPage"], "2");
        assert_eq!(json["errors"][0]["id"], "9");
        assert_eq!(json["errors"][0]["code"], "item-unreadable");
        assert_eq!(json["hits"][0]["snippet"], "the\nbudget");
    }
}
