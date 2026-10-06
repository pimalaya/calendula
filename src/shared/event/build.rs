//! # Event build
//!
//! The `event build` command: an event composed and printed, sent nowhere.

use std::{fmt, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::{
    build::{IcalBuildArgs, IcalFields},
    event::{
        Event, expand,
        fields::{EventFieldsArgs, EventMethodArg, set_method},
    },
    ical::IcalFamily,
    item::CalendarItem,
};

/// Build an event and print it, writing it nowhere.
///
/// The source, the field flags and `-i` stack as they do on `event
/// create`, and the event comes back on stdout instead of going to a
/// backend, which is how an event is judged before it is sent. No
/// account is resolved unless `-i` needs the configured composer.
///
/// The flags cover the common fields; a recurrence, an alarm, a
/// conference link and a hand-written `VTIMEZONE` are left to the
/// composer, the complete surface. `--online-meeting` writes the ask for
/// a meeting a pimdir sync engine creates, whatever backend later stores
/// the event. An event built from flags alone is checked before it is
/// printed; a source passes through as written.
///
/// Given a source alone, this is also how a loose iCalendar (an
/// invitation attached to a mail) is read the way a calendar's events
/// are: zones resolved, the end computed, people listed.
///
/// JSON output: `{"contents", "method", "events": [...]}`, the raw
/// iCalendar as text, the calendar's `METHOD` and its events shaped as
/// `event read` prints them; or `{"message"}` with `-o`.
#[derive(Debug, Parser)]
pub struct EventBuildCommand {
    #[command(flatten)]
    pub args: IcalBuildArgs,
    /// The properties the command sets on the event.
    #[command(flatten)]
    pub fields: EventFieldsArgs,
    /// Scheduling method of the calendar (`METHOD`, RFC 5546), which
    /// makes it an invitation (`request`), a cancellation (`cancel`)...
    ///
    /// Set on the calendar rather than on its event, so it holds for a
    /// source of several events too.
    #[arg(long, value_name = "METHOD")]
    pub method: Option<EventMethodArg>,
}

impl EventBuildCommand {
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
    ) -> Result<()> {
        let fields = EventBuildFields {
            fields: self.fields,
            method: self.method,
        };

        self.args.execute_with(
            printer,
            config_paths,
            account_name,
            IcalFamily::Event,
            &fields,
        )
    }
}

/// What `event build` sets between its source and the composer: the
/// event's fields, then the calendar's method.
struct EventBuildFields {
    fields: EventFieldsArgs,
    method: Option<EventMethodArg>,
}

impl IcalFields for EventBuildFields {
    fn is_empty(&self) -> bool {
        self.fields.is_empty() && self.method.is_none()
    }

    fn apply(&self, item: &[u8]) -> Result<Vec<u8>> {
        let item = self.fields.apply(item)?;

        match self.method {
            Some(method) => set_method(&item, method),
            None => Ok(item),
        }
    }
}

/// The built event, and what it projects to.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventBuildOutput {
    /// The raw iCalendar, as text.
    pub contents: String,
    /// The calendar's `METHOD` uppercased (`REQUEST`, `CANCEL`,
    /// `REPLY`...), `null` when it carries none.
    pub method: Option<String>,
    /// Every VEVENT it carries, in source order, a series once at its own
    /// start; the item id is empty, the etag `null`.
    pub events: Vec<Event>,
}

impl EventBuildOutput {
    /// Projects the built bytes.
    pub fn of(contents: String) -> Self {
        let item = CalendarItem {
            contents: contents.clone().into_bytes(),
            ..Default::default()
        };

        Self {
            method: expand::method(&item.contents),
            events: Event::project(&item),
            contents,
        }
    }
}

impl fmt::Display for EventBuildOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.contents)
    }
}

#[cfg(test)]
mod tests {
    use core::fmt;

    use anyhow::Result;
    use clap::Parser;
    use pimalaya_cli::printer::Printer;
    use serde::Serialize;
    use serde_json::Value;

    use super::EventBuildCommand;

    /// A printer keeping what it was handed, as JSON.
    #[derive(Default)]
    struct Capture(Vec<Value>);

    impl Printer for Capture {
        fn out<T: fmt::Display + Serialize>(&mut self, data: T) -> Result<()> {
            self.0.push(serde_json::to_value(data)?);
            Ok(())
        }

        fn is_json(&self) -> bool {
            true
        }
    }

    fn build(args: &[&str]) -> Result<Value> {
        let command = EventBuildCommand::try_parse_from([&["build"], args].concat())?;
        let mut printer = Capture::default();
        command.execute(&mut printer, &[], None)?;

        Ok(printer.0.remove(0))
    }

    #[test]
    fn a_build_from_flags_alone_projects_what_the_flags_say() {
        let out = build(&[
            "--uid",
            "1@example.org",
            "--summary",
            "Review",
            "--start",
            "Europe/Paris:2026-10-19T09:00",
            "--duration",
            "PT1H30M",
            "--organizer",
            "Jane Doe:jane@example.org",
            "--attendee",
            "bob@example.org",
            "--status",
            "confirmed",
            "--sequence",
            "2",
            "--method",
            "request",
        ])
        .unwrap();

        assert_eq!(out["method"], "REQUEST");

        let events = out["events"].as_array().unwrap();
        assert_eq!(events.len(), 1, "{out}");

        let event = &events[0];
        assert_eq!(event["uid"], "1@example.org");
        assert_eq!(event["summary"], "Review");
        assert_eq!(event["startsAt"], "2026-10-19T09:00:00+02:00");
        assert_eq!(event["endsAt"], "2026-10-19T10:30:00+02:00");
        assert_eq!(event["timeZone"], "Europe/Paris");
        assert_eq!(event["zoneAssumed"], false);
        assert_eq!(event["status"], "CONFIRMED");
        assert_eq!(event["sequence"], 2);
        assert_eq!(event["organizer"]["email"], "jane@example.org");
        assert_eq!(event["organizer"]["name"], "Jane Doe");
        assert_eq!(event["attendees"][0]["email"], "bob@example.org");
        assert_eq!(event["attendees"][0]["partstat"], "NEEDS-ACTION");
        assert_eq!(event["attendees"][0]["rsvp"], true);

        let contents = out["contents"].as_str().unwrap();
        assert_eq!(contents.matches("BEGIN:VTIMEZONE").count(), 1, "{contents}");
        assert!(contents.contains("DTSTAMP:"), "{contents}");
    }

    #[test]
    fn nothing_to_build_names_the_flags() {
        let err = build(&[]).unwrap_err().to_string();

        assert!(err.contains("field flag"), "{err}");
    }

    #[test]
    fn a_source_with_no_flag_comes_back_as_written() {
        let source = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:a\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let out = build(&[source]).unwrap();

        // NOTE: no PRODID, no DTSTAMP, no DTSTART: a source is not checked.
        assert_eq!(out["contents"], source);
    }

    #[test]
    fn a_build_asks_for_an_online_meeting_with_no_backend() {
        let out = build(&[
            "--summary",
            "Review",
            "--start",
            "2026-10-19T09:00Z",
            "--online-meeting",
        ])
        .unwrap();

        let contents = out["contents"].as_str().unwrap();
        assert!(
            contents.contains("X-PIMDIR-ONLINE-MEETING:TRUE\r\n"),
            "{contents}"
        );
    }

    #[test]
    fn an_empty_flag_builds_an_event_without_its_property() {
        let source = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\n\
                      BEGIN:VEVENT\r\nUID:a\r\nDTSTAMP:20260101T000000Z\r\n\
                      DTSTART:20261019T090000Z\r\nLOCATION:Room 1\r\nEND:VEVENT\r\n\
                      END:VCALENDAR\r\n";
        let out = build(&[source, "--location", ""]).unwrap();

        assert_eq!(out["contents"], source.replace("LOCATION:Room 1\r\n", ""));
        assert_eq!(out["events"][0]["location"], "");
    }
}
