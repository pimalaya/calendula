//! # Pimdir search
//!
//! The `pimdir search` command.

use std::fmt;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use io_pimdir::client::search::{PimdirSearchOptions, PimdirSearchSort};
use jiff::tz::TimeZone;
use pimalaya_cli::{
    printer::Printer,
    table::{Cell, Color, ColumnConstraint, ContentArrangement, Row, Table, TableStyle, sanitize},
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
    shared::event::list::table_times,
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
/// write is not in it. A line under the table says what it could not see,
/// another names the index's zone when it reads floating times in another.
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

    /// Maximum width of the rendered table, in terminal columns.
    #[arg(long = "max-width", short = 'w', value_name = "COLUMNS")]
    pub max_width: Option<u16>,

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
        let colors = [
            account.events_list_table_id_color(),
            account.events_list_table_summary_color(),
            account.events_list_table_start_color(),
            account.events_list_table_end_color(),
        ];

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
            table: PimdirSearchTable {
                style: account.table_style(),
                arrangement: account.table_arrangement(),
                max_width: self.max_width,
                colors,
            },
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

/// How the plain output draws its table.
#[derive(Clone, Debug)]
pub struct PimdirSearchTable {
    /// The table preset the account configures.
    pub style: TableStyle,
    /// How the table spreads its columns over the terminal.
    pub arrangement: ContentArrangement,
    /// The `--max-width` cap, when one was given.
    pub max_width: Option<u16>,
    /// The colors of the `event list` table.
    pub colors: [Color; 4],
}

/// The `pimdir search` output.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirSearchOutput {
    /// How the table renders.
    #[serde(skip)]
    pub table: PimdirSearchTable,
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
        let [id, summary, start, end] = self.table.colors;
        let zone = TimeZone::try_system().unwrap_or(TimeZone::UTC);
        let mut table = Table::new();

        table
            .load_style(self.table.style)
            .set_content_arrangement(self.table.arrangement.clone())
            .set_header(Row::from([
                "ID", "SUMMARY", "START", "END", "CALENDAR", "MATCHED", "SNIPPET",
            ]));

        for hit in &self.hits {
            let (starts, ends) = table_times(&hit.event, &zone);
            let matched = hit
                .matched
                .map(|matched| matched.to_string())
                .unwrap_or_default();
            let snippet = hit
                .snippet
                .as_deref()
                .unwrap_or_default()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");

            let mut row = Row::from([
                Cell::new(&hit.event.id).fg(id),
                Cell::new(sanitize(&hit.event.summary)).fg(summary),
                Cell::new(starts).fg(start),
                Cell::new(ends).fg(end),
                Cell::new(sanitize(&hit.calendar_id)),
                Cell::new(matched),
                Cell::new(sanitize(&snippet)),
            ]);
            row.max_height(1);
            table.add_row(row);
        }

        // NOTE: a truncated id or time is useless, so they stay whole.
        for index in [0, 2, 3] {
            if let Some(column) = table.column_mut(index) {
                column.set_constraint(ColumnConstraint::ContentWidth);
            }
        }

        if let Some(width) = self.table.max_width {
            table.set_width(width);
        }

        writeln!(f)?;
        writeln!(f, "{table}")?;
        write!(f, "{}", self.coverage)?;
        if let Some(zones) = &self.zones {
            write!(f, "{zones}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pimalaya_cli::table::{Color, ContentArrangement};

    use crate::{
        pimdir::{
            index::{
                PimdirCalendarCoverage, PimdirCalendarHit, PimdirCalendarMatched,
                PimdirCalendarZones,
            },
            search::{PimdirSearchOutput, PimdirSearchTable},
        },
        shared::{event::Event, table::style_from_preset},
    };

    #[test]
    fn an_answer_renders_the_event_columns_coverage_and_zones() {
        let output = PimdirSearchOutput {
            table: PimdirSearchTable {
                style: style_from_preset("||--+==+|-+||++++++"),
                arrangement: ContentArrangement::Disabled,
                max_width: None,
                colors: [Color::Reset; 4],
            },
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

        let shown = output.to_string();
        let lines: Vec<&str> = shown.lines().collect();

        for column in [
            "ID", "SUMMARY", "START", "END", "CALENDAR", "MATCHED", "SNIPPET",
        ] {
            assert!(lines[2].contains(column), "{shown}");
        }
        for cell in [
            "Budget review",
            "| 2026-11-10 | 2026-11-10 |",
            "cal",
            "summary",
            "the budget",
        ] {
            assert!(lines[4].contains(cell), "{shown}");
        }
        let [.., incomplete, zones] = lines.as_slice() else {
            panic!("{shown}");
        };
        assert!(
            incomplete.starts_with("Incomplete: 1 item(s) have no local body"),
            "{shown}"
        );
        assert_eq!(
            *zones,
            "The index reads floating times and dates in UTC, this search reads days in \
             Europe/Paris"
        );
    }
}
