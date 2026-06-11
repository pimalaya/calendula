use anyhow::Result;
use clap::Parser;
use io_webdav::rfc4791::calendar::Calendar;
use pimalaya_cli::printer::{Message, Printer};

use crate::caldav::client::CaldavClient;

/// Create a CalDAV calendar under the resolved calendar home-set.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct CaldavCalendarCreateCommand {
    /// Calendar identifier (last path segment of the calendar URL).
    #[arg(value_name = "ID")]
    pub id: String,

    /// Human-readable display name.
    #[arg(short, long, value_name = "NAME")]
    pub display_name: Option<String>,

    /// Free-form description.
    #[arg(short = 'D', long, value_name = "TEXT")]
    pub description: Option<String>,

    /// Hex color (`#RRGGBB`).
    #[arg(long, value_name = "HEX")]
    pub color: Option<String>,
}

impl CaldavCalendarCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CaldavClient) -> Result<()> {
        let calendar = Calendar {
            id: self.id.clone(),
            display_name: self.display_name,
            description: self.description,
            color: self.color,
            ctag: None,
            tz: None,
        };

        client.create_calendar(&calendar)?;

        let msg = format!("Calendar `{}` successfully created", self.id);
        printer.out(Message::new(msg))
    }
}
