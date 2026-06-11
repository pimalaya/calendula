use std::fmt;

use anyhow::Result;
use clap::Parser;
use comfy_table::{Cell, Color, Row, Table};
use pimalaya_cli::printer::Printer;
use serde::Serialize;

use crate::caldav::client::CaldavClient;

/// List CalDAV calendars from the resolved calendar home-set.
///
/// JSON output: `{"calendars": [{"id", "display_name", "description",
/// "color"}]}`.
#[derive(Debug, Parser)]
pub struct CaldavCalendarListCommand;

impl CaldavCalendarListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CaldavClient) -> Result<()> {
        let preset = client.account.table_preset().to_string();
        let name_color = client.account.calendars_list_table_name_color();

        let calendars = client.list_calendars()?;
        let rows: Vec<CalendarRow> = calendars
            .into_iter()
            .map(|c| CalendarRow {
                id: c.id,
                display_name: c.display_name,
                description: c.description,
                color: c.color,
            })
            .collect();

        printer.out(CalendarsTable {
            preset,
            name_color,
            rows,
        })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CalendarsTable {
    #[serde(skip)]
    pub preset: String,
    #[serde(skip)]
    pub name_color: Color,
    #[serde(rename = "calendars")]
    pub rows: Vec<CalendarRow>,
}

impl fmt::Display for CalendarsTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_preset(&self.preset)
            .set_header(Row::from([
                Cell::new("ID"),
                Cell::new("NAME"),
                Cell::new("DESCRIPTION"),
                Cell::new("COLOR"),
            ]))
            .add_rows(self.rows.iter().map(|c| {
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(&c.id))
                    .add_cell(
                        Cell::new(c.display_name.as_deref().unwrap_or("")).fg(self.name_color),
                    )
                    .add_cell(Cell::new(c.description.as_deref().unwrap_or("")))
                    .add_cell(Cell::new(c.color.as_deref().unwrap_or("")));
                row
            }));

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CalendarRow {
    pub id: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
}
