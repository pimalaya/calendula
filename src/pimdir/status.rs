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
/// every write the store's owner parked in those calendars, and every
/// event a server refused for good.
///
/// A calendar whose items list but do not read is waiting on a sync to
/// hydrate them, and this is where to see that coming. A parked write
/// never lands: it is shown by its queue row handle (ROW), which `pimdir
/// queue cancel` cancels it by, `pimdir queue list` listing every row of a
/// calendar. A refused event was applied in the store, but a server
/// refused for good the change it led to (a time its calendar does not
/// take, a value it does not store): the sync engine holds that change,
/// pushing it again only once the event is edited or removed (pimdir
/// STORAGE §15.4).
///
/// JSON output: `{"account", "accounts", "calendars": [{"id", "name",
/// "total", "hydrated", "queued"}], "parked": [{"row", "handle", "calendarId",
/// "kind", "error"}], "refused": [{"calendarId", "itemId", "source",
/// "reason", "refusedAt"}]}`, `kind` kebab-case as the queue spells it.
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
    /// Every refusal standing in those calendars, per event and source.
    pub refused: Vec<PimdirRefusedStatus>,
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
    /// The row's handle, its id under the store's identity (pimdir STORAGE
    /// §4.5), which `pimdir queue show` and `cancel` take and check.
    pub handle: String,
    /// The calendar the action targets.
    pub calendar_id: String,
    /// The action kind, as the queue spells it: `add`, `update`, `remove`,
    /// `calendar-reply`...
    pub kind: String,
    /// Why the owner parked it.
    pub error: String,
}

/// One refused event in a [`PimdirStatusOutput`] (pimdir STORAGE §15.4).
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirRefusedStatus {
    /// The calendar holding the event.
    pub calendar_id: String,
    /// The event, as `event read` takes it.
    pub item_id: String,
    /// The source whose server refused the change.
    pub source: String,
    /// The text the server gave.
    pub reason: String,
    /// When the refusal was recorded, RFC 3339.
    pub refused_at: String,
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

        if !self.parked.is_empty() {
            let mut columns = Columns::new(["ROW", "KIND", "CALENDAR", "ERROR"]);

            for parked in &self.parked {
                columns.push([
                    sanitize::text(&parked.handle).into_owned(),
                    sanitize::text(&parked.kind).into_owned(),
                    sanitize::text(&parked.calendar_id).into_owned(),
                    sanitize::text(&parked.error).into_owned(),
                ]);
            }

            writeln!(f)?;
            write!(f, "{columns}")?;
            writeln!(f, "Parked by the store's owner, never applied")?;
        }

        if !self.refused.is_empty() {
            let mut columns = Columns::new(["EVENT", "CALENDAR", "SOURCE", "REASON"]);

            for refused in &self.refused {
                columns.push([
                    sanitize::text(&refused.item_id).into_owned(),
                    sanitize::text(&refused.calendar_id).into_owned(),
                    sanitize::text(&refused.source).into_owned(),
                    sanitize::text(&refused.reason).into_owned(),
                ]);
            }

            writeln!(f)?;
            write!(f, "{columns}")?;
            writeln!(
                f,
                "Refused by the server, held until the event is edited or removed"
            )?;
        }

        Ok(())
    }
}
