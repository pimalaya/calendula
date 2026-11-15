//! # Calendar create
//!
//! The `calendar create` command, adding a calendar to the account.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{capability::Capability, shared::client::CalendarClient};

/// Create a new calendar.
///
/// The reported id is the one the backend assigned, and it is what later
/// commands address the calendar by. Given only a name, vdir names the
/// calendar's directory after it and CalDAV mints a UUID for its URL
/// segment; an id given with `--id` is kept by both. Google and Graph
/// mint their own, so they refuse `--id` with `capability-missing`
/// before connecting.
///
/// JSON output: `{"id", "name", "queued"}`, the calendar created, the name
/// it was given and whether a sync engine applies the write later.
#[derive(Debug, Parser)]
pub struct CalendarCreateCommand {
    /// Human-readable display name.
    #[arg(value_name = "NAME")]
    pub name: String,

    /// Stable calendar identifier, on the backends letting a client name a
    /// calendar (vdir, CalDAV), refused on the others.
    ///
    /// Omitted, vdir takes the name, which must then be one directory
    /// name, and CalDAV a fresh UUID.
    #[arg(long, value_name = "ID")]
    pub id: Option<String>,

    /// Free-form description.
    #[arg(short, long, value_name = "TEXT")]
    pub description: Option<String>,

    /// Hex color (`#RRGGBB`).
    #[arg(short = 'C', long, value_name = "HEX")]
    pub color: Option<String>,
}

impl CalendarCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, client: &mut CalendarClient) -> Result<()> {
        let mut capabilities = vec![Capability::CalendarCreate];
        if self.id.is_some() {
            capabilities.push(Capability::CalendarCreateId);
        }
        if self.description.is_some() {
            capabilities.push(Capability::CalendarDescription);
        }
        if self.color.is_some() {
            capabilities.push(Capability::CalendarColor);
        }
        client.require(&capabilities)?;

        let id = client.create_calendar(
            self.id.as_deref(),
            &self.name,
            self.description.as_deref(),
            self.color.as_deref(),
        )?;

        printer.out(CalendarCreateOutput {
            id,
            name: self.name,
            queued: client.queues_writes(),
        })
    }
}

/// The calendar `calendar create` made.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CalendarCreateOutput {
    /// The id the backend assigned, which later commands address the
    /// calendar by.
    pub id: String,
    /// The display name the calendar was given.
    pub name: String,
    /// Whether a sync engine applies the write later rather than now.
    pub queued: bool,
}

impl fmt::Display for CalendarCreateOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = sanitize::text(&self.id);

        match self.queued {
            true => writeln!(f, "Calendar `{id}` creation queued for the next sync"),
            false => writeln!(f, "Calendar `{id}` successfully created"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::shared::calendar::create::CalendarCreateOutput;

    #[test]
    fn a_created_calendar_prints_its_id_name_and_queued() {
        let output = CalendarCreateOutput {
            id: "work".into(),
            name: "Work".into(),
            queued: false,
        };

        assert_eq!(output.to_string(), "Calendar `work` successfully created\n");
        assert_eq!(
            serde_json::to_string(&output).unwrap(),
            r#"{"id":"work","name":"Work","queued":false}"#
        );
    }
}
