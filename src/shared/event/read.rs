//! # Event read
//!
//! The `calendula event read` command, printing one VEVENT as the
//! calendar stores it, and what it projects to, the item named by its id
//! or by the iCalendar `UID` it carries.

use std::fmt;

use anyhow::{Result, anyhow};
use clap::Parser;
use ical::component::IcalComponentKind;
use log::warn;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    capability::Capability,
    error::{event_missing, event_not_found},
    shared::{
        arg::CalendarIdArg,
        client::CalendarClient,
        event::{Event, ensure_event},
        item::{CalendarItem, CalendarItemQuery, ItemReadRow, printable},
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
/// An invitation names its event by `UID`, which is not the id a backend
/// addresses it by: `--uid` reads the item carrying it instead of an id.
/// Google Calendar is asked for the UID alone; another remote backend
/// scans the calendar whole, and a local one reads it whole. On pimdir,
/// whose stored summary names each resource's `UID`, only the bodies that
/// may carry it are read.
///
/// An id naming a resource that holds no VEVENT (a to-do, a journal
/// entry) fails `event-not-found`.
///
/// JSON output: `{"id", "calendarId", "etag", "contents", "encoding",
/// "events": [...]}`: the resource holding the event (its id, calendar,
/// version and raw iCalendar, its bytes kept, as text or as base64 when
/// they are not valid UTF-8), and its events shaped as `event list`
/// prints them: every VEVENT the resource carries, or the one occurrence
/// `--recurrence-id` names.
#[derive(Debug, Parser)]
pub struct EventReadCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// Identifier of the event, as a listing shows it.
    #[arg(
        value_name = "EVENT-ID",
        required_unless_present = "uid",
        conflicts_with = "uid"
    )]
    pub event_id: Option<String>,

    /// The iCalendar UID of the event, compared exactly, in place of its
    /// id.
    #[arg(long, value_name = "UID")]
    pub uid: Option<String>,

    /// The occurrence to project, by the `recurrenceId` an expanded
    /// listing shows (`YYYYMMDD`, `YYYYMMDDTHHMMSS` or with a `Z`).
    #[arg(long, short = 'r', value_name = "RECURRENCE-ID")]
    pub recurrence_id: Option<String>,
}

impl EventReadCommand {
    pub fn execute(self, printer: &mut impl Printer, client: &mut CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;

        let item = match &self.uid {
            Some(uid) => {
                client.require(&[Capability::EventList])?;
                read_uid(client, &calendar_id, uid)?
            }
            None => {
                let event_id = self.event_id.as_deref().unwrap_or_default();
                client.require(&[Capability::EventRead])?;
                let item = client
                    .get_item(&calendar_id, event_id)
                    .map_err(|err| event_missing(err, event_id, &calendar_id))?;
                ensure_event(&item.contents, event_id, &calendar_id)?;
                item
            }
        };

        let events = match &self.recurrence_id {
            None => Event::project(&item),
            Some(recurrence_id) => {
                let occurrence = Event::occurrence(&item, recurrence_id).ok_or_else(|| {
                    anyhow!("Event `{}` has no occurrence `{recurrence_id}`", item.id)
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

/// The item of `calendar_id` holding a VEVENT whose `UID` is `uid`,
/// refused `event-not-found` when none does. A calendar holding several,
/// which one calendar object resource should not, reads the first listed
/// and warns of the others.
fn read_uid(client: &mut CalendarClient, calendar_id: &str, uid: &str) -> Result<CalendarItem> {
    let query = CalendarItemQuery {
        kind: Some(IcalComponentKind::VEvent),
        uid: Some(uid),
        ..Default::default()
    };
    let listed = client.list_items(calendar_id, None, None, query)?;

    let carries = |item: &CalendarItem| {
        let events = Event::project(item);
        events.iter().any(|event| event.uid.as_deref() == Some(uid))
    };
    let mut items = listed.items.into_iter().filter(carries);

    let Some(item) = items.next() else {
        return Err(event_not_found(format!(
            "No event with UID `{uid}` found in calendar `{calendar_id}`"
        )));
    };

    for other in items {
        warn!("item {} carries UID {uid} too, left unread", other.id);
    }

    Ok(item)
}

/// The read event, as the calendar stores it: its resource, and its
/// events.
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
