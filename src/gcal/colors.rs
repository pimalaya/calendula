use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{
    printer::Printer,
    table::{Cell, Row, Table, TableStyle},
};
use serde::Serialize;

use crate::gcal::client::GcalClient;

/// Show the two colour palettes.
///
/// Google talks in colour ids rather than hex values, so this is what
/// turns a `colorId` into something readable, and what tells you which
/// id to reach for.
///
/// JSON output: `{"colors": [{"palette", "id", "background",
/// "foreground"}]}`.
#[derive(Debug, Parser)]
pub struct GcalColorsCommand;

impl GcalColorsCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GcalClient) -> Result<()> {
        let style = client.account.table_style();
        let palettes = client.colors_get()?.response;

        let rows = palettes
            .calendar
            .into_iter()
            .map(|entry| (entry, "calendar"))
            .chain(palettes.event.into_iter().map(|entry| (entry, "event")))
            .map(|((id, definition), palette)| ColorRow {
                palette,
                id,
                background: definition.background,
                foreground: definition.foreground,
            })
            .collect();

        printer.out(ColorsTable { style, rows })
    }
}

/// The rendered palettes.
#[derive(Clone, Debug, Serialize)]
pub struct ColorsTable {
    #[serde(skip)]
    pub style: TableStyle,
    #[serde(rename = "colors")]
    pub rows: Vec<ColorRow>,
}

/// One palette entry.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ColorRow {
    /// Which palette the entry belongs to: `calendar` or `event`.
    pub palette: &'static str,
    /// The id a `colorId` field carries.
    pub id: String,
    /// The background colour, as `#rrggbb`.
    pub background: Option<String>,
    /// The foreground colour, as `#rrggbb`.
    pub foreground: Option<String>,
}

impl fmt::Display for ColorsTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(self.style)
            .set_header(Row::from([
                Cell::new("PALETTE"),
                Cell::new("ID"),
                Cell::new("BACKGROUND"),
                Cell::new("FOREGROUND"),
            ]))
            .add_rows(self.rows.iter().map(|color| {
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(color.palette))
                    .add_cell(Cell::new(&color.id))
                    .add_cell(Cell::new(color.background.as_deref().unwrap_or("")))
                    .add_cell(Cell::new(color.foreground.as_deref().unwrap_or("")));
                row
            }));

        writeln!(f)?;
        writeln!(f, "{table}")
    }
}
