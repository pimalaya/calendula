//! # Pimdir status
//!
//! The `pimdir status` command and the report it renders.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{columns::Columns, fields::Fields, printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::pimdir::backend::PimdirBackend;

/// Report what the local pimdir store holds.
///
/// Shows the account this client reads, every account the store groups
/// collections under, one row per calendar with how many of its items
/// carry a local body and how many creations are still queued, and
/// every write the store's owner parked in those calendars.
///
/// A calendar whose items list but do not read is waiting on a sync to
/// hydrate them, and this is where to see that coming. A parked write
/// never lands: it is shown by its queue row (ROW), the id the store's
/// own operator tooling retries or cancels it by.
///
/// JSON output: `{"account", "accounts", "calendars": [{"id", "name",
/// "total", "hydrated", "queued"}], "parked": [{"row", "calendarId",
/// "action", "error"}]}`.
#[derive(Debug, Parser)]
pub struct PimdirStatusCommand;

impl PimdirStatusCommand {
    pub fn execute(self, printer: &mut impl Printer, mut backend: PimdirBackend) -> Result<()> {
        printer.out(backend.status()?)
    }
}

/// What a store holds, as `pimdir status` reports it.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirStatusOutput {
    /// The account this client reads, `None` in a store grouping none.
    pub account: Option<String>,
    /// Every account the store groups collections under.
    pub accounts: Vec<String>,
    /// One row per calendar collection.
    pub calendars: Vec<PimdirCalendarStatus>,
    /// Every action the store's owner parked in those calendars, in the
    /// order it was queued.
    pub parked: Vec<PimdirParkedStatus>,
}

/// One calendar's row in a [`PimdirStatusOutput`].
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirCalendarStatus {
    /// The collection id.
    pub id: String,
    /// The collection display name.
    pub name: String,
    /// How many live items it holds.
    pub total: usize,
    /// How many of those carry a local body.
    pub hydrated: usize,
    /// How many creations are queued for it.
    ///
    /// They carry no public id until the store's owner applies them,
    /// so they list nowhere else.
    pub queued: usize,
}

/// One parked action in a [`PimdirStatusOutput`].
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirParkedStatus {
    /// The queue row, which addresses the action rather than an item.
    pub row: i64,
    /// The calendar the action targets.
    pub calendar_id: String,
    /// The action kind (`add`, `update`, `remove`, `calendar-reply`…).
    pub action: String,
    /// Why the owner parked it.
    pub error: String,
}

impl fmt::Display for PimdirStatusOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fields = Fields::default();

        match &self.account {
            Some(account) => fields.push("Reading account", sanitize::text(account)),
            None => fields.push("Reading account", "none, the store groups no account"),
        }

        match self.accounts.as_slice() {
            [] => fields.push("Grouped accounts", "none"),
            accounts => fields.push("Grouped accounts", sanitize::text(&accounts.join(", "))),
        }

        write!(f, "{fields}")?;

        if !self.calendars.is_empty() {
            let mut columns = Columns::new(["ID", "DOWNLOADED", "QUEUED", "NAME"]);

            for calendar in &self.calendars {
                columns.push([
                    sanitize::text(&calendar.id).into_owned(),
                    format!("{}/{}", calendar.hydrated, calendar.total),
                    calendar.queued.to_string(),
                    sanitize::text(&calendar.name).into_owned(),
                ]);
            }

            writeln!(f)?;
            write!(f, "{columns}")?;
        }

        if self.parked.is_empty() {
            return Ok(());
        }

        let mut columns = Columns::new(["ROW", "ACTION", "CALENDAR", "ERROR"]);

        for parked in &self.parked {
            columns.push([
                parked.row.to_string(),
                sanitize::text(&parked.action).into_owned(),
                sanitize::text(&parked.calendar_id).into_owned(),
                sanitize::text(&parked.error).into_owned(),
            ]);
        }

        writeln!(f)?;
        write!(f, "{columns}")?;
        writeln!(f, "Parked by the store's owner, never applied")
    }
}
