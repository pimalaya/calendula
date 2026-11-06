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
            PimdirCalendarCoverage, PimdirCalendarHit, PimdirCalendarHits, PimdirCalendarZones,
            PimdirSearchKind, parse_query, search_clock,
        },
    },
    shared::event::list::table_times,
};

/// Search the account's items with the pimdir query language.
///
/// The query is pimdir SEARCH.md §8: words, "phrases", `field:value`
/// terms (`summary:`, `body:`, `location:`, `attendee:`, `status:`,
/// `priority:`, `category:`, `date:2026-11-09`, `date:2026-01..2026-03`),
/// `and`, `or`, `not` or `-`, and parentheses. One kind is searched,
/// events by default, in every calendar unless `-k` names one.
///
/// `date:` matches occurrences, a series included. Days, `today` and
/// `yesterday` are read in the local zone (`TZ`, then the system's).
///
/// The answer is the index as the sync engine last refreshed it: a queued
/// write is not in it. A line under the table says what it could not see,
/// another names the index's zone when it reads floating times in another.
///
/// JSON output: `{"hits": [{...the kind's projection, "calendarId",
/// "matched", "snippet"}], "more", "coverage": {"complete", "bodiless",
/// "unindexed", "behind", "since", "until", "filling", "horizonLeft",
/// "cut"}}`.
#[derive(Debug, Parser)]
pub struct PimdirSearchCommand {
    /// Calendar to search alone; every calendar when omitted.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR-ID")]
    pub calendar: Option<String>,

    /// Kind of item to search.
    #[arg(long, value_enum, default_value_t)]
    pub kind: PimdirSearchKind,

    /// Order of the hits.
    #[arg(long, value_enum, default_value_t)]
    pub sort: PimdirSearchSortArg,

    /// 1-indexed page number. Defaults to 1.
    #[arg(short, long, value_name = "N")]
    pub page: Option<u32>,

    /// Number of hits per page, the kind's `list` page size by default.
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
        let (page_size, colors) = match self.kind {
            PimdirSearchKind::Event => (
                account.events_list_page_size(),
                [
                    account.events_list_table_id_color(),
                    account.events_list_table_summary_color(),
                    account.events_list_table_start_color(),
                    account.events_list_table_end_color(),
                ],
            ),
            PimdirSearchKind::Todo => (
                account.todos_list_page_size(),
                [
                    account.todos_list_table_id_color(),
                    account.todos_list_table_summary_color(),
                    account.todos_list_table_due_color(),
                    account.todos_list_table_status_color(),
                ],
            ),
            PimdirSearchKind::Journal => (
                account.journals_list_page_size(),
                [
                    account.journals_list_table_id_color(),
                    account.journals_list_table_summary_color(),
                    account.journals_list_table_start_color(),
                    Color::Reset,
                ],
            ),
        };

        let limit = self.page_size.unwrap_or(page_size).max(1) as usize;
        let page = self.page.unwrap_or(1).max(1) as usize - 1;
        let options = PimdirSearchOptions {
            sort: self.sort.into(),
            limit,
            offset: page.saturating_mul(limit),
            clock: search_clock(),
        };

        let page =
            backend
                .client
                .search_items(query, self.kind, self.calendar.as_deref(), &options)?;

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
    /// The colors of the kind's `list` table, first four columns.
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
    pub hits: PimdirCalendarHits,
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
        let [id, summary, third, fourth] = self.table.colors;
        let mut table = Table::new();

        table
            .load_style(self.table.style)
            .set_content_arrangement(self.table.arrangement.clone());

        // NOTE: a truncated id or time is useless, so they stay whole.
        let whole = match &self.hits {
            PimdirCalendarHits::Events(events) => {
                let zone = TimeZone::try_system().unwrap_or(TimeZone::UTC);
                table.set_header(header(&["SUMMARY", "START", "END"]));
                table.add_rows(events.iter().map(|hit| {
                    let (start, end) = table_times(&hit.item, &zone);
                    row(
                        hit,
                        [
                            Cell::new(&hit.item.id).fg(id),
                            Cell::new(sanitize(&hit.item.summary)).fg(summary),
                            Cell::new(start).fg(third),
                            Cell::new(end).fg(fourth),
                        ],
                    )
                }));
                vec![0, 2, 3]
            }
            PimdirCalendarHits::Todos(todos) => {
                table.set_header(header(&["SUMMARY", "DUE", "STATUS", "PRIORITY", "DONE"]));
                table.add_rows(todos.iter().map(|hit| {
                    let priority = hit.item.priority.map(|priority| priority.to_string());
                    row(
                        hit,
                        [
                            Cell::new(&hit.item.id).fg(id),
                            Cell::new(sanitize(&hit.item.summary)).fg(summary),
                            Cell::new(&hit.item.due).fg(third),
                            Cell::new(&hit.item.status).fg(fourth),
                            Cell::new(priority.unwrap_or_default()),
                            Cell::new(hit.item.progress()),
                        ],
                    )
                }));
                vec![0]
            }
            PimdirCalendarHits::Journals(journals) => {
                table.set_header(header(&["SUMMARY", "DATE", "STATUS"]));
                table.add_rows(journals.iter().map(|hit| {
                    row(
                        hit,
                        [
                            Cell::new(&hit.item.id).fg(id),
                            Cell::new(sanitize(&hit.item.summary)).fg(summary),
                            Cell::new(&hit.item.start).fg(third),
                            Cell::new(&hit.item.status),
                        ],
                    )
                }));
                vec![0]
            }
        };

        for index in whole {
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

/// The header of a hit table around the kind's `columns`.
fn header(columns: &[&str]) -> Row {
    let mut header = vec!["ID"];
    header.extend_from_slice(columns);
    header.extend_from_slice(&["CALENDAR", "MATCHED", "SNIPPET"]);
    Row::from(header)
}

/// A hit's row around the kind's `cells`.
fn row<T, const N: usize>(hit: &PimdirCalendarHit<T>, cells: [Cell; N]) -> Row {
    let snippet = hit
        .snippet
        .as_deref()
        .unwrap_or_default()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let matched = hit
        .matched
        .map(|matched| matched.to_string())
        .unwrap_or_default();

    let mut row = Row::from(cells);
    row.max_height(1);
    row.add_cell(Cell::new(sanitize(&hit.calendar_id)));
    row.add_cell(Cell::new(matched));
    row.add_cell(Cell::new(sanitize(&snippet)));
    row
}

#[cfg(test)]
mod tests {
    use pimalaya_cli::table::{Color, ContentArrangement};

    use crate::{
        pimdir::{
            index::{
                PimdirCalendarCoverage, PimdirCalendarHit, PimdirCalendarHits,
                PimdirCalendarMatched, PimdirCalendarZones,
            },
            search::{PimdirSearchOutput, PimdirSearchTable},
        },
        shared::{table::style_from_preset, todo::Todo},
    };

    #[test]
    fn a_todo_answer_renders_its_family_columns_coverage_and_zones() {
        let output = PimdirSearchOutput {
            table: PimdirSearchTable {
                style: style_from_preset("||--+==+|-+||++++++"),
                arrangement: ContentArrangement::Disabled,
                max_width: None,
                colors: [Color::Reset; 4],
            },
            hits: PimdirCalendarHits::Todos(vec![PimdirCalendarHit {
                item: Todo {
                    id: "7".into(),
                    summary: "Write the budget report".into(),
                    due: "20261110T170000Z".into(),
                    status: "IN-PROCESS".into(),
                    priority: Some(1),
                    percent_complete: Some(40),
                },
                calendar_id: "cal".into(),
                matched: Some(PimdirCalendarMatched::Summary),
                snippet: Some("the\nbudget".into()),
            }]),
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
            "ID", "SUMMARY", "DUE", "STATUS", "PRIORITY", "DONE", "CALENDAR", "MATCHED", "SNIPPET",
        ] {
            assert!(lines[2].contains(column), "{shown}");
        }
        for cell in ["IN-PROCESS", "40%", "cal", "summary", "the budget"] {
            assert!(shown.contains(cell), "{shown}");
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
