//! # Event delete
//!
//! The `calendula event delete` command, removing one VEVENT from the
//! selected calendar.

use anyhow::{Result, bail};
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    capability::Capability,
    error::{self, ErrorCode, item_not_found},
    shared::{
        arg::{CalendarIdArg, SendUpdatesArg},
        client::CalendarClient,
        event::Event,
        ical::IcalFamily,
        item::{CalendarItem, CalendarItemQuery},
        receipt::{Write, WriteReceipt},
    },
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
/// JSON output: `{"id", "queued"}`.
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

    /// Whom a gcal delete mails.
    #[command(flatten)]
    pub send_updates: SendUpdatesArg,
}

impl EventDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, client: &mut CalendarClient) -> Result<()> {
        let mut capabilities = vec![Capability::ItemRemove];
        if self.if_match.is_some() {
            capabilities.push(Capability::ItemRemoveIfMatch);
        }
        if self.send_updates.send_updates.is_some() {
            capabilities.push(Capability::SchedulingSendUpdates);
        }
        client.require(&capabilities)?;

        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        let item_id = resolve(client, &calendar_id, &self.event_id)?;
        client.delete_item(
            &calendar_id,
            &item_id,
            self.if_match.as_deref(),
            self.send_updates.send_updates,
        )?;
        printer.out(WriteReceipt {
            id: item_id,
            queued: client.queues_writes(),
            what: "Event",
            write: Write::Delete,
        })
    }
}

/// The id of the item `event_id` names in `calendar_id`.
///
/// A read by id settles the common case in one call: an item whose body is
/// not local is there all the same. Only a read answering `item-not-found`
/// lists the calendar's events, as `event find` lists them, to look for the
/// id among their ids, then their `UID`s; any other failure of the read
/// (a refusal, a dropped connection) fails the command, never read as a
/// missing id.
fn resolve(client: &mut CalendarClient, calendar_id: &str, event_id: &str) -> Result<String> {
    match client.get_item(calendar_id, event_id) {
        Ok(_) => return Ok(event_id.to_owned()),
        Err(err) => match error::code(&err) {
            Some(ErrorCode::BodyPending) => return Ok(event_id.to_owned()),
            Some(ErrorCode::ItemNotFound) => {}
            _ => return Err(err),
        },
    }

    let query = CalendarItemQuery {
        kind: IcalFamily::Event.kind(),
        ..Default::default()
    };
    let listed = client.list_items(calendar_id, None, None, query)?;

    pick(&listed.items, calendar_id, event_id)
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
        [] => Err(item_not_found(format!(
            "Event `{event_id}` not found in calendar `{calendar_id}`, by id or by UID"
        ))),
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

#[cfg(all(test, feature = "msgraph"))]
mod msgraph_tests {
    use pimalaya_cli::error::{ErrorReport, with_code};

    use crate::{
        error::{self, ErrorCode},
        msgraph::backend::scripted::{answering, refusal, requests},
        shared::{client::CalendarClient, event::delete::resolve},
    };

    /// A lookup the server refuses fails the command with the refusal's
    /// code, in its failure object too, and lists nothing: a 403 is no
    /// missing id.
    #[test]
    fn a_refused_lookup_fails_coded_and_lists_nothing() {
        let denied = refusal("ErrorAccessDenied");
        let mut client = CalendarClient::over_msgraph(answering(&[(403, &denied)]));

        let err = resolve(&mut client, "cal", "E1").unwrap_err();
        let report = ErrorReport::<ErrorCode>::from(with_code(err, error::code));
        let json = serde_json::to_value(report).unwrap();
        assert_eq!(json["code"], "permission-denied", "{json}");
        assert_eq!(
            json["error"], "Read item `E1` from calendar `cal`",
            "{json}"
        );

        let mut backend = client.into_msgraph();
        let sent = requests(&mut backend);
        assert_eq!(sent.len(), 1, "{sent:?}");
        assert!(sent[0].starts_with("GET "), "{sent:?}");
    }

    /// A lookup answered 404 lists the calendar and looks for the id as a
    /// `UID`, a miss there being `item-not-found`.
    #[test]
    fn a_missing_id_falls_back_to_the_listing() {
        let missing = refusal("ErrorItemNotFound");
        let backend = answering(&[(404, &missing), (200, r#"{"value":[]}"#)]);
        let mut client = CalendarClient::over_msgraph(backend);

        let err = resolve(&mut client, "cal", "abc@example.org").unwrap_err();
        assert_eq!(error::code(&err), Some(ErrorCode::ItemNotFound), "{err:#}");

        let mut backend = client.into_msgraph();
        assert_eq!(requests(&mut backend).len(), 2);
    }
}
