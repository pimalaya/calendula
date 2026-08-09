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
    arg::CalendarIdArg, client::CalendarClient, items::CalendarTimeRange, todos::Todo,
};

/// List the todos of a calendar.
///
/// Only VTODO components are rendered; the other kinds a calendar holds
/// (VEVENT, VJOURNAL) are dropped, so use `item list` for the
/// unfiltered raw view.
///
/// Pass `--from` and `--to` (YYYY-MM-DD, both inclusive) to narrow the
/// listing to a window. A window lifts the default page-size cap, so
/// every match is returned.
///
/// JSON output: `{"todos": [{"id", "summary", "due", "status",
/// "priority", "percent-complete"}]}`.
#[derive(Debug, Parser)]
pub struct TodoListCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// 1-indexed page number. Defaults to 1.
    #[arg(short, long, value_name = "N")]
    pub page: Option<u32>,

    /// Number of items per page.
    #[arg(short = 's', long, value_name = "N")]
    pub page_size: Option<u32>,

    /// Only list todos due on or after this day (inclusive, YYYY-MM-DD).
    #[arg(long, value_name = "DATE")]
    pub from: Option<NaiveDate>,

    /// Only list todos due on or before this day (inclusive, YYYY-MM-DD).
    #[arg(long, value_name = "DATE")]
    pub to: Option<NaiveDate>,

    /// Maximum width of the rendered table, in terminal columns.
    #[arg(long = "max-width", short = 'w', value_name = "COLUMNS")]
    pub max_width: Option<u16>,
}

impl TodoListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        let range = CalendarTimeRange::from_days(self.from, self.to)?;

        // A window should return every match, so the default page-size
        // cap only applies to the unfiltered listing.
        let page_size = match range {
            Some(_) => self.page_size,
            None => self
                .page_size
                .or(Some(client.account.todos_list_page_size())),
        };

        // NOTE: the range narrows on DUE, which no backend indexes, so
        // it is applied here rather than pushed down: a server-side
        // filter is defined against a component's start and end, and a
        // todo carries neither.
        let items = client.list_items(&calendar_id, self.page, page_size, None)?;
        let todos = items
            .iter()
            .flat_map(Todo::project)
            .filter(|todo| due_within(todo, range.as_ref()))
            .collect();

        printer.out(Todos {
            style: client.account.table_style(),
            arrangement: client.account.table_arrangement(),
            max_width: self.max_width,
            colors: TodoColors {
                id: client.account.todos_list_table_id_color(),
                summary: client.account.todos_list_table_summary_color(),
                due: client.account.todos_list_table_due_color(),
                status: client.account.todos_list_table_status_color(),
            },
            todos,
        })
    }
}

/// Whether a todo's DUE falls inside `range`. A todo carrying no due
/// date is kept only when no window was asked for, so a filtered
/// listing never shows an undated task.
fn due_within(todo: &Todo, range: Option<&CalendarTimeRange>) -> bool {
    let Some(range) = range else {
        return true;
    };

    !todo.due.is_empty() && range.contains(&todo.due)
}

/// The per-column colors a todo listing renders with.
#[derive(Clone, Copy, Debug)]
struct TodoColors {
    id: Color,
    summary: Color,
    due: Color,
    status: Color,
}

/// The rendered todo listing.
#[derive(Clone, Debug, Serialize)]
pub struct Todos {
    #[serde(skip)]
    pub style: TableStyle,
    #[serde(skip)]
    pub arrangement: ContentArrangement,
    #[serde(skip)]
    pub max_width: Option<u16>,
    #[serde(skip)]
    colors: TodoColors,
    pub todos: Vec<Todo>,
}

impl fmt::Display for Todos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(self.style)
            .set_content_arrangement(self.arrangement.clone())
            .set_header(Row::from(vec![
                Cell::new("ID"),
                Cell::new("SUMMARY"),
                Cell::new("DUE"),
                Cell::new("STATUS"),
                Cell::new("PRIORITY"),
                Cell::new("DONE"),
            ]))
            .add_rows(self.todos.iter().map(|todo| {
                let mut row = Row::new();
                row.max_height(1);
                row.add_cell(Cell::new(&todo.id).fg(self.colors.id));
                row.add_cell(Cell::new(&todo.summary).fg(self.colors.summary));
                row.add_cell(Cell::new(&todo.due).fg(self.colors.due));
                row.add_cell(Cell::new(&todo.status).fg(self.colors.status));
                row.add_cell(Cell::new(
                    todo.priority
                        .map(|priority| priority.to_string())
                        .unwrap_or_default(),
                ));
                row.add_cell(Cell::new(todo.progress()));
                row
            }));

        if let Some(width) = self.max_width {
            table.set_width(width);
        }

        writeln!(f)?;
        writeln!(f, "{table}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_window_keeps_the_todos_due_inside_it_and_drops_the_undated() {
        let range = CalendarTimeRange {
            start: Some("20260801T000000Z".into()),
            end: Some("20260901T000000Z".into()),
        };

        let due = |due: &str| Todo {
            due: due.into(),
            ..Default::default()
        };

        assert!(due_within(&due("20260814T170000Z"), Some(&range)));
        assert!(!due_within(&due("20260914T170000Z"), Some(&range)));

        // An undated task has no due date to compare, so it only shows
        // in an unfiltered listing.
        assert!(!due_within(&due(""), Some(&range)));
        assert!(due_within(&due(""), None));
    }
}
