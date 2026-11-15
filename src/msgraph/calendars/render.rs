//! # Calendar rendering
//!
//! Text and JSON shapes of the calendar commands.

use core::fmt;

use io_msgraph::v1::rest::users::calendars::MsgraphCalendar;
use pimalaya_cli::{
    fields::Fields,
    records::{Record, Records},
    sanitize,
};
use schemars::JsonSchema;
use serde::Serialize;

/// A page of calendars.
///
/// The text prints each calendar by its id with its name, colour, owner and
/// whether it is the default one, while `--json` emits the raw Graph
/// calendar objects; `nextPage` is Graph's `@odata.nextLink`, lifted out
/// of the page, which `--page-token` takes back.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphCalendarsOutput {
    /// The raw Graph calendar objects.
    #[serde(rename = "calendars")]
    #[schemars(with = "Vec<serde_json::Value>")]
    pub calendars: Vec<MsgraphCalendar>,
    /// Graph's `@odata.nextLink` resuming the listing, `null` on its
    /// last page.
    pub next_page: Option<String>,
}

impl fmt::Display for MsgraphCalendarsOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut records = Records::default();

        for calendar in &self.calendars {
            let state = match calendar.is_default_calendar {
                Some(true) => vec![String::from("default")],
                _ => Vec::new(),
            };

            records.push(Record {
                title: sanitize::text(&calendar.id).into_owned(),
                state,
                fields: calendar_fields(calendar),
                body: String::new(),
            });
        }

        write!(f, "{records}")?;

        if let Some(cursor) = &self.next_page {
            if !records.items.is_empty() {
                writeln!(f)?;
            }

            writeln!(f, "Next page: {}", sanitize::text(cursor))?;
        }

        Ok(())
    }
}

/// A single calendar, emitted verbatim by `--json`.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct MsgraphCalendarOutput(
    #[schemars(with = "serde_json::Map<String, serde_json::Value>")] pub MsgraphCalendar,
);

impl fmt::Display for MsgraphCalendarOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let calendar = &self.0;
        let default = calendar.is_default_calendar == Some(true);

        let mut fields = Fields::default();
        fields.push("Id", sanitize::text(&calendar.id));
        fields.rows.extend(calendar_fields(calendar).rows);
        fields.push("Default", if default { "yes" } else { "no" });
        write!(f, "{fields}")
    }
}

/// The name, colour and owner of a calendar, sanitized, those it holds.
fn calendar_fields(calendar: &MsgraphCalendar) -> Fields<'static> {
    let name = calendar.name.as_option().map(String::as_str);
    let color = calendar
        .hex_color
        .as_deref()
        .filter(|color| !color.is_empty())
        .or(calendar.color.as_option().map(String::as_str));
    let owner = calendar
        .owner
        .as_ref()
        .and_then(|owner| owner.address.as_deref());

    let mut fields = Fields::default();
    fields.push("Name", sanitize::text(name.unwrap_or_default()));

    for (label, value) in [("Color", color), ("Owner", owner)] {
        if let Some(value) = value.filter(|value| !value.is_empty()) {
            fields.push(label, sanitize::text(value));
        }
    }

    fields
}
