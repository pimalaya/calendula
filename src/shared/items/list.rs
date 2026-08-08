use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{
    printer::Printer,
    table::{Cell, Color, ContentArrangement, Row, Table, TableStyle},
};
use serde::Serialize;

use crate::shared::{arg::CalendarIdArg, client::CalendarClient, items::CalendarItem};

/// List the raw iCalendar items of a calendar.
///
/// Every component kind is listed, VEVENT included; use `event list`
/// for the events-only view with its summary and time columns.
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
        let items = client.list_items(&calendar_id, self.page, page_size, None)?;

        printer.out(Items {
            style: client.account.table_style(),
            arrangement: client.account.table_arrangement(),
            max_width: self.max_width,
            colors: ItemColors {
                id: client.account.items_list_table_id_color(),
                etag: client.account.items_list_table_etag_color(),
                size: client.account.items_list_table_size_color(),
            },
            items,
        })
    }
}

/// The per-column colors an item listing renders with.
#[derive(Clone, Copy, Debug)]
struct ItemColors {
    id: Color,
    etag: Color,
    size: Color,
}

/// The rendered item listing.
#[derive(Clone, Debug, Serialize)]
pub struct Items {
    #[serde(skip)]
    pub style: TableStyle,
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
            .load_style(self.style)
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
                row.add_cell(Cell::new(size_of(item)).fg(self.colors.size));
                row
            }));

        if let Some(width) = self.max_width {
            table.set_width(width);
        }

        writeln!(f)?;
        writeln!(f, "{table}")
    }
}

/// The size column: the item's octets, or a dash when the backend
/// listed the item without a local body (a pimdir cache that has not
/// downloaded it yet).
fn size_of(item: &CalendarItem) -> String {
    if item.contents.is_empty() {
        return String::from("-");
    }

    humansize::format_size(item.contents.len() as u64, humansize::BINARY)
}
