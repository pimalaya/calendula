//! # Event read
//!
//! The `calendula event read` command, printing one VEVENT as the
//! calendar stores it, and what it projects to.

use std::fmt;

use anyhow::{Result, anyhow};
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::{arg::CalendarIdArg, client::CalendarClient, event::Event};

/// Read a single event (raw iCalendar bytes).
///
/// The bytes print as the calendar stores them, a series with every
/// override it carries. With `--recurrence-id`, the one occurrence it
/// names is projected as well, at its own times, an override applied.
///
/// JSON output: `{"contents", "events": [...]}`, the raw iCalendar and
/// its events shaped as `event list` prints them: every VEVENT the item
/// carries, or the one occurrence `--recurrence-id` names.
#[derive(Debug, Parser)]
pub struct EventReadCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// Identifier of the event, as a listing shows it.
    #[arg(value_name = "EVENT-ID")]
    pub event_id: String,

    /// The occurrence to project, by the `recurrenceId` an expanded
    /// listing shows (`YYYYMMDD`, `YYYYMMDDTHHMMSS` or with a `Z`).
    #[arg(long, short = 'r', value_name = "RECURRENCE-ID")]
    pub recurrence_id: Option<String>,
}

impl EventReadCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        let item = client.get_item(&calendar_id, &self.event_id)?;

        let events = match &self.recurrence_id {
            None => Event::project(&item),
            Some(recurrence_id) => {
                let occurrence = Event::occurrence(&item, recurrence_id).ok_or_else(|| {
                    anyhow!(
                        "Event `{}` has no occurrence `{recurrence_id}`",
                        self.event_id
                    )
                })?;
                vec![occurrence]
            }
        };

        let contents = String::from_utf8_lossy(&item.contents).into_owned();
        printer.out(EventReadOutput { contents, events })
    }
}

/// The read event, as the calendar stores it.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventReadOutput {
    /// The raw iCalendar bytes, lossily decoded as UTF-8.
    pub contents: String,
    /// What the bytes project to, or the one occurrence asked for.
    pub events: Vec<Event>,
}

impl fmt::Display for EventReadOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.contents.trim_end())
    }
}
