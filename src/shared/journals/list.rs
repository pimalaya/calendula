use std::fmt;

use anyhow::Result;
use chrono::NaiveDate;
use clap::Parser;
use pimalaya_cli::{
    printer::Printer,
    table::{Cell, Color, ContentArrangement, Row, Table, TableStyle},
};
use serde::Serialize;

use crate::shared::{
    arg::CalendarIdArg, client::CalendarClient, items::CalendarTimeRange, journals::Journal,
};

/// List the journal entries of a calendar.
///
/// Only VJOURNAL components are rendered; the other kinds a calendar
/// holds (VEVENT, VTODO) are dropped, so use `item list` for the
/// unfiltered raw view.
///
/// Pass `--from` and `--to` (YYYY-MM-DD, both inclusive) to narrow the
/// listing to a window. A window lifts the default page-size cap, so
/// every match is returned.
///
/// JSON output: `{"journals": [{"id", "summary", "start", "status"}]}`.
#[derive(Debug, Parser)]
pub struct JournalListCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// 1-indexed page number. Defaults to 1.
    #[arg(short, long, value_name = "N")]
    pub page: Option<u32>,

    /// Number of items per page.
    #[arg(short = 's', long, value_name = "N")]
    pub page_size: Option<u32>,

    /// Only list entries dated on or after this day (inclusive,
    /// YYYY-MM-DD).
    #[arg(long, value_name = "DATE")]
    pub from: Option<NaiveDate>,

    /// Only list entries dated on or before this day (inclusive,
    /// YYYY-MM-DD).
    #[arg(long, value_name = "DATE")]
    pub to: Option<NaiveDate>,

    /// Maximum width of the rendered table, in terminal columns.
    #[arg(long = "max-width", short = 'w', value_name = "COLUMNS")]
    pub max_width: Option<u16>,
}

impl JournalListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        let range = CalendarTimeRange::from_days(self.from, self.to)?;

        // A window should return every match, so the default page-size
        // cap only applies to the unfiltered listing.
        let page_size = match range {
            Some(_) => self.page_size,
            None => self
                .page_size
                .or(Some(client.account.journals_list_page_size())),
        };

        // NOTE: a server-side range filter is defined against a
        // component's start and end (RFC 4791 9.9), and a journal entry
        // carries no end, so the window is applied after parsing rather
        // than pushed down.
        let items = client.list_items(&calendar_id, self.page, page_size, None)?;
        let journals = items
            .iter()
            .flat_map(Journal::project)
            .filter(|journal| dated_within(journal, range.as_ref()))
            .collect();

        printer.out(Journals {
            style: client.account.table_style(),
            arrangement: client.account.table_arrangement(),
            max_width: self.max_width,
            colors: JournalColors {
                id: client.account.journals_list_table_id_color(),
                summary: client.account.journals_list_table_summary_color(),
                start: client.account.journals_list_table_start_color(),
            },
            journals,
        })
    }
}

/// Whether an entry's date falls inside `range`. An entry carrying no
/// date is kept only when no window was asked for.
fn dated_within(journal: &Journal, range: Option<&CalendarTimeRange>) -> bool {
    let Some(range) = range else {
        return true;
    };

    !journal.start.is_empty() && range.contains(&journal.start)
}

/// The per-column colors a journal listing renders with.
#[derive(Clone, Copy, Debug)]
struct JournalColors {
    id: Color,
    summary: Color,
    start: Color,
}

/// The rendered journal listing.
#[derive(Clone, Debug, Serialize)]
pub struct Journals {
    #[serde(skip)]
    pub style: TableStyle,
    #[serde(skip)]
    pub arrangement: ContentArrangement,
    #[serde(skip)]
    pub max_width: Option<u16>,
    #[serde(skip)]
    colors: JournalColors,
    pub journals: Vec<Journal>,
}

impl fmt::Display for Journals {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(self.style)
            .set_content_arrangement(self.arrangement.clone())
            .set_header(Row::from(vec![
                Cell::new("ID"),
                Cell::new("SUMMARY"),
                Cell::new("DATE"),
                Cell::new("STATUS"),
            ]))
            .add_rows(self.journals.iter().map(|journal| {
                let mut row = Row::new();
                row.max_height(1);
                row.add_cell(Cell::new(&journal.id).fg(self.colors.id));
                row.add_cell(Cell::new(&journal.summary).fg(self.colors.summary));
                row.add_cell(Cell::new(&journal.start).fg(self.colors.start));
                row.add_cell(Cell::new(&journal.status));
                row
            }));

        if let Some(width) = self.max_width {
            table.set_width(width);
        }

        writeln!(f)?;
        writeln!(f, "{table}")
    }
}
