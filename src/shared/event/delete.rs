//! # Event delete
//!
//! The `calendula event delete` command, removing one VEVENT from the
//! selected calendar.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    capability::Capability,
    error::{self, ErrorCode, event_missing},
    shared::{
        arg::{CalendarIdArg, SendUpdatesArg},
        client::CalendarClient,
        event::ensure_event,
        receipt::{Write, WriteReceipt},
    },
};

/// Delete a single event.
///
/// Immediate and unconditional: the event is removed with no
/// confirmation prompt and no copy kept. A series goes whole, with
/// every override it carries.
///
/// The event is named by the id `event list` shows, never by its
/// iCalendar `UID`, which `event read --uid` reads it by.
///
/// JSON output: `{"id", "queued"}`.
#[derive(Debug, Parser)]
pub struct EventDeleteCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// Event to delete: its id, as `event list` reports it.
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
        let mut capabilities = vec![Capability::EventDelete];
        if self.if_match.is_some() {
            capabilities.push(Capability::EventDeleteIfMatch);
        }
        if self.send_updates.send_updates.is_some() {
            capabilities.push(Capability::SchedulingSendUpdates);
        }
        client.require(&capabilities)?;

        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        check(client, &calendar_id, &self.event_id)?;
        client
            .delete_item(
                &calendar_id,
                &self.event_id,
                self.if_match.as_deref(),
                self.send_updates.send_updates,
            )
            .map_err(|err| event_missing(err, &self.event_id, &calendar_id))?;
        printer.out(WriteReceipt {
            id: self.event_id,
            queued: client.queues_writes(),
            what: "Event",
            write: Write::Delete,
        })
    }
}

/// Fails unless `event_id` names an item of `calendar_id` holding a VEVENT.
///
/// A read by id settles it in one call: an item whose body is not local is
/// there all the same, its kind left to the summary it was listed by. A
/// read answering `event-not-found` is a miss, never looked for as a `UID`,
/// which `event read --uid` reads by; any other failure of the read (a
/// refusal, a dropped connection) fails the command with its code.
pub fn check(client: &mut CalendarClient, calendar_id: &str, event_id: &str) -> Result<()> {
    let err = match client.get_item(calendar_id, event_id) {
        Ok(item) => return ensure_event(&item.contents, event_id, calendar_id),
        Err(err) => err,
    };

    match error::code(&err) {
        Some(ErrorCode::BodyPending) => Ok(()),
        Some(ErrorCode::EventNotFound) => Err(event_missing(err, event_id, calendar_id)),
        _ => Err(err),
    }
}

#[cfg(all(test, feature = "msgraph"))]
mod msgraph_tests {
    use pimalaya_cli::error::{ErrorReport, with_code};

    use crate::{
        error::{self, ErrorCode},
        msgraph::backend::scripted::{answering, refusal, requests},
        shared::{client::CalendarClient, event::delete::check},
    };

    /// A lookup the server refuses fails the command with the refusal's
    /// code, in its failure object too, and lists nothing: a 403 is no
    /// missing id.
    #[test]
    fn a_refused_lookup_fails_coded_and_lists_nothing() {
        let denied = refusal("ErrorAccessDenied");
        let mut client = CalendarClient::over_msgraph(answering(&[(403, &denied)]));

        let err = check(&mut client, "cal", "E1").unwrap_err();
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

    /// A lookup answered 404 is `event-not-found`, pointing at `event
    /// search`, and lists nothing: a UID is no id.
    #[test]
    fn a_missing_id_is_not_found_and_lists_nothing() {
        let missing = refusal("ErrorItemNotFound");
        let backend = answering(&[(404, &missing), (200, r#"{"value":[]}"#)]);
        let mut client = CalendarClient::over_msgraph(backend);

        let err = check(&mut client, "cal", "abc@example.org").unwrap_err();
        assert_eq!(error::code(&err), Some(ErrorCode::EventNotFound), "{err:#}");
        assert!(
            err.to_string().contains("`event read --uid <UID>`"),
            "{err}"
        );

        let mut backend = client.into_msgraph();
        assert_eq!(requests(&mut backend).len(), 1);
    }
}
