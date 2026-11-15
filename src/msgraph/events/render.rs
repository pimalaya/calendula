//! # Event rendering
//!
//! Text and JSON shapes of the event commands.

use core::fmt;

use io_msgraph::v1::rest::users::events::{MsgraphDateTimeTimeZone, MsgraphEvent};
use pimalaya_cli::{
    fields::Fields,
    records::{Record, Records},
    sanitize,
};
use schemars::JsonSchema;
use serde::Serialize;

/// A page of events.
///
/// The text prints each event by its id with its subject, start, end,
/// location and organizer, while `--json` emits the raw Graph event
/// objects plus any next-page link.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphEventsOutput {
    /// The raw Graph event objects.
    #[serde(rename = "events")]
    #[schemars(with = "Vec<serde_json::Value>")]
    pub events: Vec<MsgraphEvent>,
    /// The link to the next page, when the page was truncated.
    #[serde(rename = "@odata.nextLink", skip_serializing_if = "Option::is_none")]
    pub next_link: Option<String>,
}

impl fmt::Display for MsgraphEventsOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut records = Records::default();

        for event in &self.events {
            records.push(Record {
                title: sanitize::text(&event.id).into_owned(),
                state: Vec::new(),
                fields: event_fields(event),
                body: String::new(),
            });
        }

        write!(f, "{records}")?;

        if self.next_link.is_some() {
            if !records.items.is_empty() {
                writeln!(f)?;
            }

            writeln!(f, "(more events available: raise --top)")?;
        }

        Ok(())
    }
}

/// A single event, emitted verbatim by `--json`.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct MsgraphEventOutput(
    #[schemars(with = "serde_json::Map<String, serde_json::Value>")] pub MsgraphEvent,
);

impl fmt::Display for MsgraphEventOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let event = &self.0;
        let mut fields = Fields::default();
        fields.push("Id", sanitize::text(&event.id));
        fields.rows.extend(event_fields(event).rows);
        write!(f, "{fields}")
    }
}

/// The subject, start, end, location and organizer of an event,
/// sanitized, those it holds.
pub fn event_fields(event: &MsgraphEvent) -> Fields<'static> {
    let subject = event.subject.as_option().map(String::as_str);
    let start = event.start.as_option().map(date_time);
    let end = event.end.as_option().map(date_time);
    let location = event
        .location
        .as_option()
        .and_then(|location| location.display_name.as_deref());
    let organizer = event
        .organizer
        .as_option()
        .and_then(|organizer| organizer.email_address.address.as_deref());

    let mut fields = Fields::default();
    let rows = [
        ("Subject", subject),
        ("Start", start.as_deref()),
        ("End", end.as_deref()),
        ("Location", location),
        ("Organizer", organizer),
    ];

    for (label, value) in rows {
        if let Some(value) = value.filter(|value| !value.is_empty()) {
            fields.push(label, sanitize::text(value));
        }
    }

    fields
}

/// A Graph date-time as it spells it, its zone after it.
fn date_time(stamp: &MsgraphDateTimeTimeZone) -> String {
    match &stamp.time_zone {
        Some(zone) => format!("{} {zone}", stamp.date_time),
        None => stamp.date_time.clone(),
    }
}

#[cfg(test)]
mod tests {
    use io_msgraph::v1::rest::users::events::MsgraphEvent;

    use super::MsgraphEventsOutput;

    /// An event prints the fields it holds, no empty one.
    #[test]
    fn an_event_prints_the_fields_it_holds() {
        let event: MsgraphEvent = serde_json::from_str(
            r#"{"id":"E1","subject":"Standup",
            "start":{"dateTime":"2026-11-16T09:00:00.0000000","timeZone":"UTC"},
            "end":{"dateTime":"2026-11-16T09:15:00.0000000","timeZone":"UTC"}}"#,
        )
        .unwrap();
        let output = MsgraphEventsOutput {
            events: vec![event],
            next_link: None,
        };

        let expected = "\
E1
Subject:  Standup
Start:    2026-11-16T09:00:00.0000000 UTC
End:      2026-11-16T09:15:00.0000000 UTC
";
        assert_eq!(output.to_string(), expected);
    }
}
