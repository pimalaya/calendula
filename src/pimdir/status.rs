//! # Pimdir status
//!
//! The `pimdir status` command and the report it renders.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
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
        writeln!(f)?;

        match &self.account {
            Some(account) => writeln!(f, "Reading account: {account}")?,
            None => writeln!(f, "Reading account: none, the store groups no account")?,
        }

        match self.accounts.as_slice() {
            [] => writeln!(f, "Grouped accounts: none")?,
            accounts => writeln!(f, "Grouped accounts: {}", accounts.join(", "))?,
        }

        writeln!(f)?;

        for calendar in &self.calendars {
            write!(
                f,
                "  {} ({}): {}/{} items downloaded",
                calendar.name, calendar.id, calendar.hydrated, calendar.total
            )?;

            match calendar.queued {
                0 => writeln!(f)?,
                queued => writeln!(f, ", {queued} queued for the next sync")?,
            }
        }

        if self.parked.is_empty() {
            return Ok(());
        }

        writeln!(f)?;
        writeln!(f, "Parked, never applied:")?;

        for parked in &self.parked {
            writeln!(
                f,
                "  ROW {} {} in {}: {}",
                parked.row, parked.action, parked.calendar_id, parked.error
            )?;
        }

        Ok(())
    }
}
