//! # Pimdir search
//!
//! The `pimdir search` command.

use std::fmt;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use io_pimdir::client::search::{PimdirSearchOptions, PimdirSearchSort};
use jiff::tz::TimeZone;
use pimalaya_cli::{
    fields::Fields,
    printer::Printer,
    records::{Record, Records},
    sanitize,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    pimdir::{
        backend::PimdirBackend,
        index::{
            PimdirCalendarCoverage, PimdirCalendarHit, PimdirCalendarZones, parse_query,
            search_clock,
        },
    },
    shared::event::list::{local_zone, row_times},
};

/// Search the account's events with the pimdir query language.
///
/// The query is pimdir SEARCH.md §8: words, "phrases", `field:value`
/// terms (`summary:`, `body:`, `location:`, `attendee:`, `organizer:`,
/// `category:`, `date:2026-11-09`, `date:2026-01..2026-03`), `and`, `or`,
/// `not` or `-`, and parentheses. Every calendar is searched unless `-k`
/// names one.
///
/// `date:` matches occurrences, a series included. Days, `today` and
/// `yesterday` are read in the local zone (`TZ`, then the system's).
///
/// The answer is the index as the sync engine last refreshed it: a queued
/// write is not in it. Each hit prints as a record, its summary then its
/// snippet as the body; a line under the hits says what the answer could
/// not see, another names the index's zone when it reads floating times
/// in another.
///
/// JSON output: `{"hits": [{...the event projection, "calendarId",
/// "matched", "snippet"}], "more", "coverage": {"complete", "bodiless",
/// "unindexed", "behind", "since", "until", "filling", "horizonLeft",
/// "cut"}}`.
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

    /// The query, its words joined by spaces.
    #[arg(value_name = "QUERY", required = true)]
    #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
    pub query: Vec<String>,
}

impl PimdirSearchCommand {
    pub fn execute(self, printer: &mut impl Printer, backend: PimdirBackend) -> Result<()> {
        let query = parse_query(&self.query, printer.is_json())?;
        if let Some(calendar) = &self.calendar {
            backend.known_collection(calendar)?;
        }

        let account = &backend.account;
        let page_size = self.page_size.unwrap_or(account.events_list_page_size());
        let limit = page_size.max(1) as usize;
        let page = self.page.unwrap_or(1).max(1) as usize - 1;
        let options = PimdirSearchOptions {
            sort: self.sort.into(),
            limit,
            offset: page.saturating_mul(limit),
            clock: search_clock(),
        };

        let page = backend
            .client
            .search_items(query, self.calendar.as_deref(), &options)?;

        printer.out(PimdirSearchOutput {
            zone: local_zone(),
            hits: page.hits,
            more: page.more,
            coverage: page.coverage,
            zones: page.zones,
        })
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
    /// The zone a timed hit's start and end print in.
    #[serde(skip)]
    pub zone: TimeZone,
    /// The hits of this page.
    pub hits: Vec<PimdirCalendarHit>,
    /// Whether hits follow this page.
    pub more: bool,
    /// What the answer could not see.
    pub coverage: PimdirCalendarCoverage,
    /// The zones, when the index reads floating times in another.
    #[serde(skip)]
    pub zones: Option<PimdirCalendarZones>,
}

impl fmt::Display for PimdirSearchOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut records = Records::default();

        for hit in &self.hits {
            let (start, end) = row_times(&hit.event, &self.zone);

            let mut fields = Fields::default();
            fields.push("Start", sanitize::text(&start));
            fields.push("End", sanitize::text(&end));
            fields.push("Calendar", sanitize::text(&hit.calendar_id));

            if let Some(matched) = hit.matched {
                fields.push("Matched", matched);
            }

            let snippet = one_line(hit.snippet.as_deref());
            let body = [sanitize::text(&hit.event.summary), sanitize::text(&snippet)]
                .into_iter()
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join("\n\n");

            records.push(Record {
                title: sanitize::text(&hit.event.id).into_owned(),
                state: Vec::new(),
                fields,
                body,
            });
        }

        let mut notes = self.coverage.to_string();

        if let Some(zones) = &self.zones {
            notes.push_str(&zones.to_string());
        }

        write!(f, "{records}")?;

        if !records.items.is_empty() && !notes.is_empty() {
            writeln!(f)?;
        }

        f.write_str(&notes)
    }
}

/// A passage on one line, its whitespace runs collapsed.
fn one_line(text: Option<&str>) -> String {
    let words = text.unwrap_or_default().split_whitespace();
    words.collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use jiff::tz::TimeZone;

    use crate::{
        pimdir::{
            index::{
                PimdirCalendarCoverage, PimdirCalendarHit, PimdirCalendarMatched,
                PimdirCalendarZones,
            },
            search::PimdirSearchOutput,
        },
        shared::event::Event,
    };

    /// A hit prints as a record, its summary then its snippet as the body,
    /// and what the answer could not see a blank line under the records.
    #[test]
    fn hits_print_as_records_above_the_coverage_and_zones() {
        let output = PimdirSearchOutput {
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
            more: false,
            coverage: PimdirCalendarCoverage {
                bodiless: 1,
                ..Default::default()
            },
            zones: Some(PimdirCalendarZones {
                index: "UTC".into(),
                search: "Europe/Paris".into(),
            }),
        };

        let expected = "\
7
Start:     2026-11-10
End:       2026-11-10
Calendar:  cal
Matched:   summary

    Budget review

    the budget

Incomplete: 1 item(s) have no local body and matched on their summary alone
The index reads floating times and dates in UTC, this search reads days in Europe/Paris
";
        assert_eq!(output.to_string(), expected);
    }
}
