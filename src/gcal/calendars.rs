use std::fmt;

use anyhow::Result;
use clap::Parser;
use io_gcal::v3::rest::calendar_list::list::GcalCalendarListListParams;
use pimalaya_cli::{
    printer::Printer,
    table::{Cell, Color, Row, Table, TableStyle},
};
use serde::Serialize;

use crate::gcal::{client::GcalClient, render};

/// List the calendars of the user's calendar list.
///
/// Unlike the shared listing, this shows what only Google carries: the
/// access role the account has on each calendar, which one is the
/// primary, the time zone events are expanded in, and the default
/// reminders a new event inherits.
///
/// JSON output: `{"calendars": [{"id", "summary", "description",
/// "time-zone", "access-role", "primary", "selected", "color",
/// "default-reminders"}]}`.
#[derive(Debug, Parser)]
pub struct GcalCalendarListCommand {
    /// Include the calendars hidden from the list.
    #[arg(long)]
    pub show_hidden: bool,
}

impl GcalCalendarListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GcalClient) -> Result<()> {
        let style = client.account.table_style();
        let name_color = client.account.calendars_list_table_name_color();

        let mut rows = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let params = GcalCalendarListListParams {
                page_token: page_token.as_deref(),
                show_hidden: self.show_hidden,
                ..Default::default()
            };
            let page = client.calendar_list_list(&params)?.response;

            rows.extend(page.items.into_iter().map(|entry| {
                CalendarRow {
                    id: entry.id.unwrap_or_default(),
                    summary: entry.summary_override.or(entry.summary).unwrap_or_default(),
                    description: entry.description,
                    time_zone: entry.time_zone,
                    access_role: entry.access_role.map(render::access_role),
                    primary: entry.primary.unwrap_or(false),
                    selected: entry.selected.unwrap_or(false),
                    color: entry.background_color,
                    default_reminders: entry
                        .default_reminders
                        .iter()
                        .filter_map(|reminder| Some(format!("{}m", reminder.minutes?)))
                        .collect(),
                }
            }));

            match page.next_page_token {
                Some(next) => page_token = Some(next),
                None => break,
            }
        }

        printer.out(CalendarsTable {
            style,
            name_color,
            rows,
        })
    }
}

/// The rendered Google calendar listing.
#[derive(Clone, Debug, Serialize)]
pub struct CalendarsTable {
    #[serde(skip)]
    pub style: TableStyle,
    #[serde(skip)]
    pub name_color: Color,
    #[serde(rename = "calendars")]
    pub rows: Vec<CalendarRow>,
}

/// One calendar list entry, with the properties only Google carries.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct CalendarRow {
    /// The calendar identifier, an address for a primary or secondary
    /// calendar.
    pub id: String,
    /// The title, the one this user gave the calendar winning over the
    /// calendar's own.
    pub summary: String,
    /// The free-form description.
    pub description: Option<String>,
    /// The IANA time zone the calendar expands its events in.
    pub time_zone: Option<String>,
    /// The effective access role this account has: `owner`, `writer`,
    /// `reader` or `freeBusyReader`.
    pub access_role: Option<&'static str>,
    /// Whether this is the account's own primary calendar.
    pub primary: bool,
    /// Whether the calendar's contents show in the Google UI.
    pub selected: bool,
    /// The background colour, as `#rrggbb`.
    pub color: Option<String>,
    /// The default reminder lead times a new event inherits.
    pub default_reminders: Vec<String>,
}

impl fmt::Display for CalendarsTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(self.style)
            .set_header(Row::from([
                Cell::new("ID"),
                Cell::new("NAME"),
                Cell::new("ROLE"),
                Cell::new("TIMEZONE"),
                Cell::new("PRIMARY"),
                Cell::new("REMINDERS"),
            ]))
            .add_rows(self.rows.iter().map(|calendar| {
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(&calendar.id))
                    .add_cell(Cell::new(&calendar.summary).fg(self.name_color))
                    .add_cell(Cell::new(calendar.access_role.unwrap_or("")))
                    .add_cell(Cell::new(calendar.time_zone.as_deref().unwrap_or("")))
                    .add_cell(Cell::new(if calendar.primary { "yes" } else { "" }))
                    .add_cell(Cell::new(calendar.default_reminders.join(", ")));
                row
            }));

        writeln!(f)?;
        writeln!(f, "{table}")
    }
}
