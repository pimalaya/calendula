use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::client::CalendarClient;

/// Create a new calendar.
///
/// The reported id is the one the backend assigned. It is the id given
/// here on every backend that lets a client name a collection; Google
/// mints its own, so there it is the minted one, which is what later
/// commands address the calendar by.
///
/// JSON output: `{"message": "..."}`.
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
        let id = client.create_calendar(
            &self.id,
            &self.name,
            self.description.as_deref(),
            self.color.as_deref(),
        )?;

        let msg = format!("Calendar `{id}` successfully created");
        printer.out(Message::new(msg))
    }
}
