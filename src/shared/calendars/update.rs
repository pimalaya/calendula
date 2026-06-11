use anyhow::Result;
use clap::Parser;
use io_calendar::calendar::CalendarDiff;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::{arg::CalendarIdArg, client::CalendarClient};

/// Update a calendar's mutable properties.
///
/// Each `--*` flag is optional and only updates the corresponding
/// field; unset fields are left untouched. To clear an optional field,
/// pass an empty value (e.g. `--description ""`).
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct CalendarUpdateCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// New display name.
    #[arg(short, long, value_name = "NAME")]
    pub name: Option<String>,

    /// New description. Pass an empty string to clear.
    #[arg(short, long, value_name = "TEXT")]
    pub description: Option<String>,

    /// New hex color (`#RRGGBB`). Pass an empty string to clear.
    #[arg(long, value_name = "HEX")]
    pub color: Option<String>,
}

impl CalendarUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let patch = CalendarDiff {
            name: self.name,
            description: self
                .description
                .map(|s| if s.is_empty() { None } else { Some(s) }),
            color: self
                .color
                .map(|s| if s.is_empty() { None } else { Some(s) }),
        };

        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        client.update_calendar(&calendar_id, patch)?;

        let msg = format!("Calendar `{calendar_id}` successfully updated");
        printer.out(Message::new(msg))
    }
}
