use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{
    printer::Printer,
    table::{Cell, Color, ContentArrangement, Row, Table, TableStyle},
};
use serde::Serialize;

use crate::shared::{calendars::Calendar, client::CalendarClient};

/// List the calendars of the active account.
///
/// Every backend serves this the same way; use the protocol-specific
/// listings when you need what only one backend exposes.
///
/// JSON output: `{"calendars": [{"id", "name", "description",
/// "color"}]}`.
#[derive(Debug, Parser)]
pub struct CalendarListCommand {
    /// Maximum width of the rendered table, in terminal columns.
    #[arg(long = "max-width", short = 'w')]
    #[arg(value_name = "COLUMNS")]
    pub max_width: Option<u16>,
}

impl CalendarListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendars = client.list_calendars()?;

        printer.out(Calendars {
            style: client.account.table_style(),
            arrangement: client.account.table_arrangement(),
            max_width: self.max_width,
            colors: CalendarColors {
                id: client.account.calendars_list_table_id_color(),
                name: client.account.calendars_list_table_name_color(),
                description: client.account.calendars_list_table_description_color(),
                color: client.account.calendars_list_table_color_color(),
            },
            calendars,
        })
    }
}

/// The per-column colors a calendar listing renders with.
#[derive(Clone, Copy, Debug)]
struct CalendarColors {
    id: Color,
    name: Color,
    description: Color,
    color: Color,
}

/// The rendered calendar listing.
#[derive(Clone, Debug, Serialize)]
pub struct Calendars {
    #[serde(skip)]
    pub style: TableStyle,
    #[serde(skip)]
    pub arrangement: ContentArrangement,
    #[serde(skip)]
    pub max_width: Option<u16>,
    #[serde(skip)]
    colors: CalendarColors,
    pub calendars: Vec<Calendar>,
}

impl fmt::Display for Calendars {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(self.style)
            .set_content_arrangement(self.arrangement.clone())
            .set_header(Row::from(vec![
                Cell::new("ID"),
                Cell::new("NAME"),
                Cell::new("DESCRIPTION"),
                Cell::new("COLOR"),
            ]))
            .add_rows(self.calendars.iter().map(|calendar| {
                let mut row = Row::new();
                row.max_height(1);
                row.add_cell(Cell::new(&calendar.id).fg(self.colors.id));
                row.add_cell(Cell::new(&calendar.name).fg(self.colors.name));
                row.add_cell(
                    Cell::new(calendar.description.as_deref().unwrap_or(""))
                        .fg(self.colors.description),
                );
                row.add_cell(
                    Cell::new(calendar.color.as_deref().unwrap_or("")).fg(self.colors.color),
                );
                row
            }));

        if let Some(width) = self.max_width {
            table.set_width(width);
        }

        writeln!(f)?;
        writeln!(f, "{table}")
    }
}
