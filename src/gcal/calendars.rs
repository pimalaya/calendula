//! # Calendar list
//!
//! The `calendula gcal calendars` command, listing the user's calendar list
//! with the properties only Google carries.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use io_gcal::v3::rest::calendar_list::list::GcalCalendarListListParams;
use pimalaya_cli::{
    fields::Fields,
    printer::Printer,
    records::{Record, Records},
    sanitize,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    gcal::{client::GcalClient, render},
    shared::calendar::list::body,
};

/// List the calendars of the user's calendar list.
///
/// Unlike the shared listing, this shows what only Google carries: the
/// access role the account has on each calendar, which one is the primary,
/// the time zone events expand in, and the default reminders a new event
/// inherits.
///
/// JSON output: `{"calendars": [{"id", "summary", "description",
/// "timeZone", "accessRole", "primary", "selected", "color",
/// "defaultReminders"}]}`.
#[derive(Debug, Parser)]
pub struct GcalCalendarListCommand {
    /// Include the calendars hidden from the list.
    #[arg(long)]
    pub show_hidden: bool,
}

impl GcalCalendarListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GcalClient) -> Result<()> {
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
                    access_role: entry
                        .access_role
                        .as_ref()
                        .map(|role| render::access_role(role).to_owned()),
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

        printer.out(GcalCalendarListOutput { rows })
    }
}

/// The Google calendar listing.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GcalCalendarListOutput {
    /// The calendars of the list, one entry each.
    #[serde(rename = "calendars")]
    pub rows: Vec<CalendarRow>,
}

/// One calendar list entry, with the properties only Google carries.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CalendarRow {
    /// The calendar identifier, an address for a primary or secondary one.
    pub id: String,
    /// The title, this user's own overriding the calendar's.
    pub summary: String,
    /// The free-form description.
    pub description: Option<String>,
    /// The IANA time zone the calendar expands its events in.
    pub time_zone: Option<String>,
    /// The effective access role this account has: `owner`, `writer`,
    /// `reader` or `freeBusyReader`.
    pub access_role: Option<String>,
    /// Whether this is the account's own primary calendar.
    pub primary: bool,
    /// Whether the calendar's contents show in the Google UI.
    pub selected: bool,
    /// The background colour, as `#rrggbb`.
    pub color: Option<String>,
    /// The default reminder lead times a new event inherits.
    pub default_reminders: Vec<String>,
}

impl fmt::Display for GcalCalendarListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut records = Records::default();

        for calendar in &self.rows {
            let mut fields = Fields::default();
            fields.push("Name", sanitize::text(&calendar.summary));

            if let Some(role) = &calendar.access_role {
                fields.push("Role", sanitize::text(role));
            }

            let optional = [
                ("Time zone", calendar.time_zone.as_deref()),
                ("Color", calendar.color.as_deref()),
            ];

            for (label, value) in optional {
                if let Some(value) = value {
                    fields.push(label, sanitize::text(value));
                }
            }

            if !calendar.default_reminders.is_empty() {
                fields.push("Reminders", calendar.default_reminders.join(", "));
            }

            let state = match calendar.primary {
                true => vec![String::from("primary")],
                false => Vec::new(),
            };

            let description = calendar.description.as_deref().unwrap_or_default();

            records.push(Record {
                title: sanitize::text(&calendar.id).into_owned(),
                state,
                fields,
                body: body(description),
            });
        }

        write!(f, "{records}")
    }
}
