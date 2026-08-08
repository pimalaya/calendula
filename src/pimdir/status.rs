//! The `pimdir status` command and the report it renders.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use serde::Serialize;

use crate::pimdir::backend::PimdirBackend;

/// Report what the local pimdir store holds.
///
/// Shows the source writes are attributed to, every source the store
/// has been synced as, and one row per calendar with how many of its
/// items carry a local body. A calendar whose items list but do not
/// read is waiting on a sync to hydrate them, and this is where to see
/// that coming.
///
/// JSON output: `{"source", "sources", "calendars": [{"id", "name",
/// "total", "hydrated"}]}`.
#[derive(Debug, Parser)]
pub struct PimdirStatusCommand;

impl PimdirStatusCommand {
    pub fn execute(self, printer: &mut impl Printer, mut backend: PimdirBackend) -> Result<()> {
        printer.out(backend.status()?)
    }
}

/// What a store holds, as `pimdir status` reports it.
#[derive(Clone, Debug, Serialize)]
pub struct PimdirStatus {
    /// The source this account's writes are attributed to.
    pub source: String,
    /// Every source the store has been synced as.
    pub sources: Vec<String>,
    /// One row per calendar collection.
    pub calendars: Vec<PimdirCalendarStatus>,
}

/// One calendar's row in a [`PimdirStatus`].
#[derive(Clone, Debug, Serialize)]
pub struct PimdirCalendarStatus {
    /// The collection id.
    pub id: String,
    /// The collection display name.
    pub name: String,
    /// How many live items it holds.
    pub total: usize,
    /// How many of those carry a local body.
    pub hydrated: usize,
}

impl fmt::Display for PimdirStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        writeln!(f, "Writes attributed to source: {}", self.source)?;

        match self.sources.as_slice() {
            [] => writeln!(f, "Synced sources: none yet")?,
            sources => writeln!(f, "Synced sources: {}", sources.join(", "))?,
        }

        writeln!(f)?;

        for calendar in &self.calendars {
            writeln!(
                f,
                "  {} ({}): {}/{} items downloaded",
                calendar.name, calendar.id, calendar.hydrated, calendar.total
            )?;
        }

        Ok(())
    }
}
