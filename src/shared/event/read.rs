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

use crate::{
    capability::Capability,
    error::event_missing,
    shared::{
        arg::CalendarIdArg,
        client::CalendarClient,
        event::Event,
        item::{ItemReadRow, read::printable},
    },
};

/// Read a single event (its iCalendar).
///
/// The iCalendar prints as the calendar stores it, a series with every
/// override it carries, CRLF read as LF and any character that could
/// drive the terminal replaced. With `--recurrence-id`, the one
/// occurrence it names is projected as well, at its own times, an
/// override applied.
///
/// JSON output: `{"id", "calendarId", "etag", "contents", "encoding",
/// "events": [...]}`, the row `item list` prints of the item holding the
/// event (its raw iCalendar, its bytes kept, as text or as base64 when
/// they are not valid UTF-8), and its events shaped as `event list`
/// prints them: every VEVENT the item carries, or the one occurrence
/// `--recurrence-id` names.
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
    pub fn execute(self, printer: &mut impl Printer, client: &mut CalendarClient) -> Result<()> {
        client.require(&[Capability::ItemRead])?;

        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        let item = client
            .get_item(&calendar_id, &self.event_id)
            .map_err(|err| event_missing(err, &self.event_id, &calendar_id))?;

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

        printer.out(EventReadOutput {
            row: ItemReadRow::from(&item),
            events,
            bytes: item.contents,
        })
    }
}

/// The read event, as the calendar stores it: the row `item list` prints
/// of its item, and its events.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventReadOutput {
    /// The item holding the event.
    #[serde(flatten)]
    pub row: ItemReadRow,
    /// What the bytes project to, or the one occurrence asked for.
    pub events: Vec<Event>,
    /// The bytes read, which the text prints.
    #[serde(skip)]
    pub bytes: Vec<u8>,
}

impl fmt::Display for EventReadOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", printable(&self.bytes).trim_end())
    }
}
