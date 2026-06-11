use std::fmt::{self, Write};

use anyhow::Result;
use clap::Parser;
use comfy_table::{Cell, Color, ContentArrangement, Row, Table};
use io_calendar::{
    calcard::{
        common::PartialDateTime,
        icalendar::{
            ICalendarComponent, ICalendarComponentType, ICalendarProperty, ICalendarValue,
        },
    },
    item::CalendarItem,
};
use pimalaya_cli::printer::Printer;
use serde::Serialize;

use crate::shared::{arg::CalendarIdArg, client::CalendarClient};

/// List VEVENT items inside a calendar.
///
/// Non-VEVENT items (VTODO, VJOURNAL) are filtered out of the
/// rendering; use `items list` for the unfiltered raw view.
///
/// JSON output: `{"events": [{"id", "summary", "start", "end"}]}`.
#[derive(Debug, Parser)]
pub struct EventListCommand {
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

impl EventListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        let page_size = self
            .page_size
            .or(Some(client.account.events_list_page_size()));
        let raw_items = client.list_items(&calendar_id, self.page, page_size)?;

        let events: Vec<EventRow> = raw_items
            .into_iter()
            .filter_map(extract_event_row)
            .collect();

        let table = Events {
            preset: client.account.table_preset().to_string(),
            arrangement: client.account.table_arrangement(),
            max_width: self.max_width,
            colors: EventColors {
                id: client.account.events_list_table_id_color(),
                summary: client.account.events_list_table_summary_color(),
                start: client.account.events_list_table_start_color(),
                end: client.account.events_list_table_end_color(),
            },
            events,
        };

        printer.out(table)
    }
}

#[derive(Clone, Copy, Debug)]
struct EventColors {
    id: Color,
    summary: Color,
    start: Color,
    end: Color,
}

#[derive(Clone, Debug, Serialize)]
pub struct EventRow {
    pub id: String,
    pub summary: String,
    pub start: String,
    pub end: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Events {
    #[serde(skip)]
    pub preset: String,
    #[serde(skip)]
    pub arrangement: ContentArrangement,
    #[serde(skip)]
    pub max_width: Option<u16>,
    #[serde(skip)]
    colors: EventColors,
    pub events: Vec<EventRow>,
}

impl fmt::Display for Events {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_preset(&self.preset)
            .set_content_arrangement(self.arrangement.clone())
            .set_header(Row::from(vec![
                Cell::new("ID"),
                Cell::new("SUMMARY"),
                Cell::new("START"),
                Cell::new("END"),
            ]))
            .add_rows(self.events.iter().map(|e| {
                let mut row = Row::new();
                row.max_height(1);
                row.add_cell(Cell::new(&e.id).fg(self.colors.id));
                row.add_cell(Cell::new(&e.summary).fg(self.colors.summary));
                row.add_cell(Cell::new(&e.start).fg(self.colors.start));
                row.add_cell(Cell::new(&e.end).fg(self.colors.end));
                row
            }));

        if let Some(width) = self.max_width {
            table.set_width(width);
        }

        writeln!(f)?;
        writeln!(f, "{table}")
    }
}

/// Parses `item` and pulls out the first VEVENT component's SUMMARY,
/// DTSTART, and DTEND. Returns [`None`] when the item is not a
/// recognisable VEVENT.
fn extract_event_row(item: CalendarItem) -> Option<EventRow> {
    let ical = item.as_ical()?;
    let vevent = ical
        .components
        .iter()
        .find(|c| c.component_type == ICalendarComponentType::VEvent)?;

    let summary = component_text(vevent, &ICalendarProperty::Summary).unwrap_or_default();
    let start = component_date(vevent, &ICalendarProperty::Dtstart).unwrap_or_default();
    let end = component_date(vevent, &ICalendarProperty::Dtend).unwrap_or_default();

    Some(EventRow {
        id: item.id,
        summary,
        start,
        end,
    })
}

fn component_text(component: &ICalendarComponent, name: &ICalendarProperty) -> Option<String> {
    let prop = component.property(name)?;
    prop.values.iter().find_map(|v| match v {
        ICalendarValue::Text(text) => Some(text.clone()),
        _ => None,
    })
}

fn component_date(component: &ICalendarComponent, name: &ICalendarProperty) -> Option<String> {
    let prop = component.property(name)?;
    prop.values.iter().find_map(|v| match v {
        ICalendarValue::PartialDateTime(pdt) => Some(format_partial_datetime(pdt)),
        _ => None,
    })
}

/// Renders a [`PartialDateTime`] as a YYYYMMDD or YYYYMMDDTHHMMSS
/// string, mirroring the iCalendar wire format used by upstream
/// consumers of `events list`.
fn format_partial_datetime(pdt: &PartialDateTime) -> String {
    let mut out = String::with_capacity(16);

    if let (Some(y), Some(m), Some(d)) = (pdt.year, pdt.month, pdt.day) {
        let _ = write!(&mut out, "{y:04}{m:02}{d:02}");
    }

    if let (Some(h), Some(min), Some(s)) = (pdt.hour, pdt.minute, pdt.second) {
        let _ = write!(&mut out, "T{h:02}{min:02}{s:02}");
    }

    out
}
