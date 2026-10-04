//! # Event delete
//!
//! The `calendula event delete` command, removing one VEVENT from the
//! selected calendar.

use anyhow::{Result, bail};
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::{
    arg::CalendarIdArg,
    client::CalendarClient,
    event::Event,
    ical::IcalFamily,
    item::{CalendarItem, CalendarItemQuery},
};

/// Delete a single event.
///
/// Immediate and unconditional: the event is removed with no
/// confirmation prompt and no copy kept. A series goes whole, with
/// every override it carries.
///
/// The event is named by the id `event list` shows or by its iCalendar
/// `UID`. The id is tried first; a `UID` carried by several items of the
/// calendar is refused, naming their ids.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct EventDeleteCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// Event to delete: its id, as `event list` reports it, or its
    /// iCalendar `UID`.
    #[arg(value_name = "EVENT-ID")]
    pub event_id: String,

    /// Gate the delete on this ETag, as `event list` or `event read`
    /// reports it.
    ///
    /// The delete only lands if the backend still holds that version
    /// (RFC 9110 `If-Match`), which is how a concurrent write is caught.
    #[arg(long, value_name = "ETAG")]
    pub if_match: Option<String>,
}

impl EventDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        let item_id = resolve(&mut client, &calendar_id, &self.event_id)?;
        client.delete_item(&calendar_id, &item_id, self.if_match.as_deref())?;
        printer.out(Message::new("Event successfully deleted"))
    }
}

/// The id of the item `event_id` names in `calendar_id`.
///
/// A read by id settles the common case in one call. Failing that, the
/// calendar's events are listed, as `event find` lists them, and the id
/// is looked for among their ids, then their `UID`s: an item whose body
/// is not local still lists under its id.
fn resolve(client: &mut CalendarClient, calendar_id: &str, event_id: &str) -> Result<String> {
    if client.get_item(calendar_id, event_id).is_ok() {
        return Ok(event_id.to_owned());
    }

    let items = client.list_items(
        calendar_id,
        CalendarItemQuery {
            kind: IcalFamily::Event.kind(),
            ..Default::default()
        },
    )?;

    pick(&items, calendar_id, event_id)
}

/// The item among `items` whose id is `event_id`, else the one item
/// carrying it as a `UID`.
fn pick(items: &[CalendarItem], calendar_id: &str, event_id: &str) -> Result<String> {
    if items.iter().any(|item| item.id == event_id) {
        return Ok(event_id.to_owned());
    }

    let carrying: Vec<&str> = items
        .iter()
        .filter(|item| {
            Event::project(item)
                .iter()
                .any(|event| event.uid == event_id)
        })
        .map(|item| item.id.as_str())
        .collect();

    match carrying.as_slice() {
        [id] => Ok((*id).to_owned()),
        [] => bail!("Event `{event_id}` not found in calendar `{calendar_id}`, by id or by UID"),
        ids => bail!(
            "Several events of calendar `{calendar_id}` carry the UID `{event_id}` ({}): \
             delete one by its id",
            ids.join(", "),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, uid: &str) -> CalendarItem {
        let contents = format!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//t//t//EN\r\nBEGIN:VEVENT\r\n\
             UID:{uid}\r\nDTSTAMP:20260101T000000Z\r\nDTSTART:20261005T090000Z\r\n\
             SUMMARY:Meeting\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
        );
        CalendarItem {
            id: id.into(),
            calendar_id: "cal".into(),
            etag: None,
            contents: contents.into_bytes(),
        }
    }

    #[test]
    fn an_id_wins_over_a_uid() {
        let items = [item("7", "abc@example.org"), item("8", "7")];
        assert_eq!(pick(&items, "cal", "7").unwrap(), "7");
    }

    #[test]
    fn a_uid_names_the_one_item_carrying_it() {
        let items = [item("7", "abc@example.org"), item("8", "def@example.org")];
        assert_eq!(pick(&items, "cal", "def@example.org").unwrap(), "8");
    }

    #[test]
    fn a_uid_carried_twice_is_refused_naming_both() {
        let items = [item("7", "abc@example.org"), item("8", "abc@example.org")];
        let err = pick(&items, "cal", "abc@example.org").unwrap_err();
        assert!(err.to_string().contains("(7, 8)"), "{err}");
    }

    #[test]
    fn nothing_matching_is_a_clear_miss() {
        let items = [item("7", "abc@example.org")];
        let err = pick(&items, "cal", "nope").unwrap_err();
        assert!(err.to_string().contains("not found"), "{err}");
    }
}
