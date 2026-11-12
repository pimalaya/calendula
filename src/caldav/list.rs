//! # CalDAV calendar list
//!
//! The `calendula caldav list` command, showing the collections under
//! the resolved home-set with the properties only CalDAV carries.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{
    fields::Fields,
    printer::Printer,
    records::{Record, Records},
    sanitize,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{caldav::client::CaldavClient, shared::calendar::list::body};

/// List the CalDAV calendars under the resolved home-set.
///
/// Unlike the shared listing, this shows what only CalDAV carries: the
/// collection change tag, the sync token an incremental sync starts
/// from, and the component kinds the server accepts.
///
/// JSON output: `{"calendars": [{"id", "displayName", "description",
/// "color", "components", "ctag", "syncToken"}]}`.
#[derive(Debug, Parser)]
pub struct CaldavCalendarListCommand;

impl CaldavCalendarListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CaldavClient) -> Result<()> {
        let rows = client
            .list_calendars()?
            .into_iter()
            .map(|calendar| CalendarRow {
                id: calendar.id,
                display_name: calendar.display_name,
                description: calendar.description,
                color: calendar.color,
                components: calendar.components.into_iter().collect(),
                ctag: calendar.ctag,
                sync_token: calendar.sync_token,
            })
            .collect();

        printer.out(CaldavCalendarListOutput { rows })
    }
}

/// The CalDAV calendar listing.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CaldavCalendarListOutput {
    /// One entry per calendar collection.
    #[serde(rename = "calendars")]
    pub rows: Vec<CalendarRow>,
}

/// One CalDAV calendar with the properties only CalDAV carries.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CalendarRow {
    /// The last path segment of the collection URL.
    pub id: String,
    /// The DAV display name.
    pub display_name: Option<String>,
    /// The free-form description (RFC 4791 6.2.1).
    pub description: Option<String>,
    /// The display color (RFC 7986 5.9).
    pub color: Option<String>,
    /// The component kinds the collection accepts (RFC 4791 5.2.3).
    ///
    /// Empty means the server declares no restriction.
    pub components: Vec<String>,
    /// The collection change tag, bumped on every change.
    pub ctag: Option<String>,
    /// The RFC 6578 sync token an incremental sync starts from.
    pub sync_token: Option<String>,
}

impl fmt::Display for CaldavCalendarListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut records = Records::default();

        for calendar in &self.rows {
            let mut fields = Fields::default();
            let name = calendar.display_name.as_deref().unwrap_or_default();
            fields.push("Name", sanitize::text(name));

            let optional = [
                ("Color", calendar.color.as_deref()),
                ("Ctag", calendar.ctag.as_deref()),
                ("Sync token", calendar.sync_token.as_deref()),
            ];

            for (label, value) in optional {
                if let Some(value) = value {
                    fields.push(label, sanitize::text(value));
                }
            }

            if !calendar.components.is_empty() {
                let components = calendar.components.join(", ");
                fields.push("Components", sanitize::text(&components));
            }

            let description = calendar.description.as_deref().unwrap_or_default();

            records.push(Record {
                title: sanitize::text(&calendar.id).into_owned(),
                state: Vec::new(),
                fields,
                body: body(description),
            });
        }

        write!(f, "{records}")
    }
}
