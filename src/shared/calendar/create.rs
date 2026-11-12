//! # Calendar create
//!
//! The `calendar create` command, adding a calendar to the account.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    capability::Capability,
    shared::{
        client::CalendarClient,
        receipt::{Write, WriteReceipt},
    },
};

/// Create a new calendar.
///
/// The reported id is the one the backend assigned, and it is what later
/// commands address the calendar by. Every backend that lets a client name a
/// collection keeps the id given here; Google and Graph mint their own.
///
/// JSON output: `{"id", "queued"}`.
#[derive(Debug, Parser)]
pub struct CalendarCreateCommand {
    /// Stable calendar identifier.
    #[arg(value_name = "ID")]
    pub id: String,

    /// Human-readable display name.
    #[arg(value_name = "NAME")]
    pub name: String,

    /// Free-form description.
    #[arg(short, long, value_name = "TEXT")]
    pub description: Option<String>,

    /// Hex color (`#RRGGBB`).
    #[arg(long, value_name = "HEX")]
    pub color: Option<String>,
}

impl CalendarCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let mut capabilities = vec![Capability::CalendarCreate];
        if self.description.is_some() {
            capabilities.push(Capability::CalendarDescription);
        }
        if self.color.is_some() {
            capabilities.push(Capability::CalendarColor);
        }
        client.require(&capabilities)?;

        let id = client.create_calendar(
            &self.id,
            &self.name,
            self.description.as_deref(),
            self.color.as_deref(),
        )?;

        printer.out(WriteReceipt {
            id,
            queued: client.queues_writes(),
            what: "Calendar",
            write: Write::Create,
        })
    }
}
