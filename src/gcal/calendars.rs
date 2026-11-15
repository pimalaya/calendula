//! # Calendar list
//!
//! The `calendula gcal calendars` command, listing the user's calendar list
//! with the properties only Google carries.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use io_gcal::v3::rest::calendar_list::list::GcalCalendarListListParams;
use pimalaya_cli::{
    fields::Fields,
    printer::Printer,
    records::{Record, Records},
    sanitize,
};
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::{
    gcal::{client::GcalClient, render::str_of},
    shared::calendar::list::body,
};

/// List the calendars of the user's calendar list.
///
/// Unlike the shared listing, this shows what only Google carries: the
/// access role the account has on each calendar, which one is the primary,
/// the time zone events expand in, and the default reminders a new event
/// inherits.
///
/// JSON output: `{"calendars": [<calendar list entry>...]}`, every entry
/// of every page as Google sent it (`id`, `summary`, `accessRole`,
/// `primary`, `defaultReminders` and the rest).
#[derive(Debug, Parser)]
pub struct GcalCalendarListCommand {
    /// Include the calendars hidden from the list.
    #[arg(long)]
    pub show_hidden: bool,
}

impl GcalCalendarListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GcalClient) -> Result<()> {
        let params = GcalCalendarListListParams {
            show_hidden: self.show_hidden,
            ..Default::default()
        };
        let calendars = client.raw_items(&["users", "me", "calendarList"], &params)?;

        printer.out(GcalCalendarListOutput { calendars })
    }
}

/// The Google calendar listing.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GcalCalendarListOutput {
    /// The entries of the calendar list, each as Google sent it.
    #[schemars(with = "Vec<Map<String, Value>>")]
    pub calendars: Vec<Value>,
}

impl fmt::Display for GcalCalendarListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut records = Records::default();

        for calendar in &self.calendars {
            let name = match str_of(calendar, "summaryOverride") {
                "" => str_of(calendar, "summary"),
                name => name,
            };

            let mut fields = Fields::default();
            fields.push("Name", sanitize::text(name));

            let optional = [
                ("Role", "accessRole"),
                ("Time zone", "timeZone"),
                ("Color", "backgroundColor"),
            ];

            for (label, key) in optional {
                if let Some(value) = calendar.get(key).and_then(Value::as_str) {
                    fields.push(label, sanitize::text(value));
                }
            }

            let reminders: Vec<String> = calendar
                .get("defaultReminders")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|reminder| reminder.get("minutes")?.as_i64())
                .map(|minutes| format!("{minutes}m"))
                .collect();

            if !reminders.is_empty() {
                fields.push("Reminders", reminders.join(", "));
            }

            let state = match calendar.get("primary").and_then(Value::as_bool) {
                Some(true) => vec![String::from("primary")],
                _ => Vec::new(),
            };

            records.push(Record {
                title: sanitize::text(str_of(calendar, "id")).into_owned(),
                state,
                fields,
                body: body(str_of(calendar, "description")),
            });
        }

        write!(f, "{records}")
    }
}

#[cfg(test)]
mod tests {
    use io_gcal::v3::client::GcalClientStd;
    use pimalaya_cli::printer::CapturePrinter;
    use serde_json::json;

    use super::*;
    use crate::{
        account::context::Account,
        script::{self, Script},
    };

    /// Every entry of every page prints as Google sent it, a key calendula
    /// never read (`conferenceProperties`) kept, the second page asked by
    /// the first's `nextPageToken`; the text reads the same entries.
    #[test]
    fn every_entry_prints_as_google_sent_it() {
        let work = json!({
            "kind": "calendar#calendarListEntry",
            "id": "work@example.org",
            "summary": "Work",
            "summaryOverride": "Job",
            "accessRole": "owner",
            "primary": true,
            "defaultReminders": [{"method": "popup", "minutes": 10}],
            "conferenceProperties": {"allowedConferenceSolutionTypes": ["hangoutsMeet"]},
        });
        let home = json!({"id": "home@example.org", "summary": "Home"});
        let first = json!({"items": [work], "nextPageToken": "p2"}).to_string();
        let second = json!({"items": [home]}).to_string();

        let (script, requests) = Script::new([
            script::response(200, "application/json", &first),
            script::response(200, "application/json", &second),
        ]);
        let client = GcalClient::new(
            GcalClientStd::new(script, "made-up"),
            "perso".into(),
            Account::default(),
        );

        let mut printer = CapturePrinter::default();
        let command = GcalCalendarListCommand { show_hidden: false };
        command.execute(&mut printer, client).unwrap();

        let json = printer.into_single().unwrap();
        assert_eq!(json, json!({"calendars": [work, home]}));

        let requests = requests.lock().unwrap();
        let second = String::from_utf8_lossy(&requests[1]);
        assert!(second.contains("pageToken=p2"), "{second}");

        let output = GcalCalendarListOutput {
            calendars: vec![work, home],
        };
        let text = output.to_string();
        assert!(text.contains("Job"), "{text}");
        assert!(text.contains("10m"), "{text}");
        assert!(text.contains("primary"), "{text}");
    }
}
