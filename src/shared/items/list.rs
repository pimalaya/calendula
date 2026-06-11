use std::fmt;

use anyhow::Result;
use clap::Parser;
use comfy_table::{Cell, Color, ContentArrangement, Row, Table};
use io_calendar::item::CalendarItem;
use pimalaya_cli::printer::Printer;
use serde::Serialize;

use crate::shared::{arg::CalendarIdArg, client::CalendarClient};

/// Shared API to list iCalendar items for a calendar.
///
/// JSON output: `{"items": [{"id", "calendar-id", "etag",
/// "contents"}]}`.
#[derive(Debug, Parser)]
pub struct ItemListCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// 1-indexed page number. Defaults to 1.
    #[arg(short, long, value_name = "N")]
    pub page: Option<u32>,

    /// Number of items per page.
    #[arg(short = 's', long, value_name = "N")]
    pub page_size: Option<u32>,

    /// Maximum width of the rendered table, in terminal columns.
    #[arg(long = "max-width", short = 'w', value_name = "COLUMNS")]
    pub max_width: Option<u16>,
}

impl ItemListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        let page_size = self
            .page_size
            .or(Some(client.account.items_list_page_size()));
        let items = client.list_items(&calendar_id, self.page, page_size)?;

        let table = Items {
            preset: client.account.table_preset().to_string(),
            arrangement: client.account.table_arrangement(),
            max_width: self.max_width,
            colors: ItemColors {
                id: client.account.items_list_table_id_color(),
                etag: client.account.items_list_table_etag_color(),
                size: client.account.items_list_table_size_color(),
            },
            items,
        };

        printer.out(table)
    }
}

#[derive(Clone, Copy, Debug)]
struct ItemColors {
    id: Color,
    etag: Color,
    size: Color,
}

#[derive(Clone, Debug, Serialize)]
pub struct Items {
    #[serde(skip)]
    pub preset: String,
    #[serde(skip)]
    pub arrangement: ContentArrangement,
    #[serde(skip)]
    pub max_width: Option<u16>,
    #[serde(skip)]
    colors: ItemColors,
    pub items: Vec<CalendarItem>,
}

impl fmt::Display for Items {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_preset(&self.preset)
            .set_content_arrangement(self.arrangement.clone())
            .set_header(Row::from(vec![
                Cell::new("ID"),
                Cell::new("ETAG"),
                Cell::new("SIZE"),
            ]))
            .add_rows(self.items.iter().map(|item| {
                let mut row = Row::new();
                row.max_height(1);
                row.add_cell(Cell::new(&item.id).fg(self.colors.id));
                row.add_cell(Cell::new(item.etag.as_deref().unwrap_or("")).fg(self.colors.etag));
                row.add_cell(
                    Cell::new(humansize::format_size(
                        item.contents.len() as u64,
                        humansize::BINARY,
                    ))
                    .fg(self.colors.size),
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
