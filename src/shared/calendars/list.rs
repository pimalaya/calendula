use std::fmt;

use anyhow::Result;
use clap::Parser;
use comfy_table::{Cell, Color, ContentArrangement, Row, Table};
use io_calendar::calendar::Calendar;
use pimalaya_cli::printer::Printer;
use serde::Serialize;

use crate::shared::client::CalendarClient;

/// Shared API to list calendars for the active account.
///
/// JSON output: `{"calendars": [{"id", "name", "description", "color",
/// "ctag"}]}`.
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

        let table = Calendars {
            preset: client.account.table_preset().to_string(),
            arrangement: client.account.table_arrangement(),
            max_width: self.max_width,
            colors: CalendarColors {
                id: client.account.calendars_list_table_id_color(),
                name: client.account.calendars_list_table_name_color(),
                description: client.account.calendars_list_table_description_color(),
                color: client.account.calendars_list_table_color_color(),
            },
            calendars,
        };

        printer.out(table)
    }
}

#[derive(Clone, Copy, Debug)]
struct CalendarColors {
    id: Color,
    name: Color,
    description: Color,
    color: Color,
}

#[derive(Clone, Debug, Serialize)]
pub struct Calendars {
    #[serde(skip)]
    pub preset: String,
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
            .load_preset(&self.preset)
            .set_content_arrangement(self.arrangement.clone())
            .set_header(Row::from(vec![
                Cell::new("ID"),
                Cell::new("NAME"),
                Cell::new("DESCRIPTION"),
                Cell::new("COLOR"),
            ]))
            .add_rows(self.calendars.iter().map(|c| {
                let mut row = Row::new();
                row.max_height(1);
                row.add_cell(Cell::new(&c.id).fg(self.colors.id));
                row.add_cell(Cell::new(&c.name).fg(self.colors.name));
                row.add_cell(
                    Cell::new(c.description.as_deref().unwrap_or("")).fg(self.colors.description),
                );
                row.add_cell(Cell::new(c.color.as_deref().unwrap_or("")).fg(self.colors.color));
                row
            }));

        if let Some(width) = self.max_width {
            table.set_width(width);
        }

        writeln!(f)?;
        writeln!(f, "{table}")
    }
}
