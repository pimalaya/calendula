use std::fmt;

use anyhow::Result;
use clap::Parser;
use io_vdir::collection::VdirCollection;
use pimalaya_cli::printer::Printer;
use pimalaya_cli::table::{Cell, Color, Row, Table, TableStyle};
use serde::Serialize;

use crate::vdir::client::VdirClient;

/// List on-disk vdir collections under the configured home directory.
///
/// JSON output: `{"collections": [{"id", "display_name", "path"}]}`.
#[derive(Debug, Parser)]
pub struct VdirCollectionListCommand;

impl VdirCollectionListCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let collections = client.list_collections()?;

        let table = CollectionsTable {
            style: client.account.table_style(),
            name_color: client.account.calendars_list_table_name_color(),
            rows: collections.into_iter().map(CollectionRow::from).collect(),
        };

        printer.out(table)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CollectionsTable {
    #[serde(skip)]
    pub style: TableStyle,
    #[serde(skip)]
    pub name_color: Color,
    #[serde(rename = "collections")]
    pub rows: Vec<CollectionRow>,
}

impl fmt::Display for CollectionsTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(self.style)
            .set_header(Row::from([
                Cell::new("ID"),
                Cell::new("NAME"),
                Cell::new("PATH"),
            ]))
            .add_rows(self.rows.iter().map(|c| {
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(&c.id))
                    .add_cell(
                        Cell::new(c.display_name.as_deref().unwrap_or("")).fg(self.name_color),
                    )
                    .add_cell(Cell::new(&c.path));
                row
            }));

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CollectionRow {
    pub id: String,
    pub display_name: Option<String>,
    pub path: String,
}

impl From<VdirCollection> for CollectionRow {
    fn from(collection: VdirCollection) -> Self {
        Self {
            id: collection.id().to_owned(),
            display_name: collection.display_name.clone(),
            path: collection.path.as_str().to_owned(),
        }
    }
}
