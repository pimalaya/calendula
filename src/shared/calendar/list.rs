//! # Calendar list
//!
//! The `calendar list` command, printing the account's calendars as
//! records.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{
    fields::Fields,
    printer::Printer,
    records::{Record, Records},
    sanitize,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::{calendar::Calendar, client::CalendarClient};

/// List the calendars of the active account.
///
/// Every backend serves this the same way; use the protocol-specific listings
/// when you need what only one backend exposes.
///
/// JSON output: `{"calendars": [{"id", "name", "description", "color",
/// "default"}]}`.
#[derive(Debug, Parser)]
pub struct CalendarListCommand;

impl CalendarListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let calendars = client.list_calendars()?;
        printer.out(CalendarListOutput { calendars })
    }
}

/// The calendar listing.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CalendarListOutput {
    /// The calendars of the account.
    pub calendars: Vec<Calendar>,
}

impl fmt::Display for CalendarListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut records = Records::default();

        for calendar in &self.calendars {
            let mut fields = Fields::default();
            fields.push("Name", sanitize::text(&calendar.name));

            if let Some(color) = &calendar.color {
                fields.push("Color", sanitize::text(color));
            }

            let state = match calendar.default {
                true => vec![String::from("default")],
                false => Vec::new(),
            };

            let description = calendar.description.as_deref().unwrap_or_default();

            records.push(Record {
                title: sanitize::text(&calendar.id).into_owned(),
                state,
                fields,
                body: body(description),
            });
        }

        write!(f, "{records}")
    }
}

/// A free text as a record body, each line defused.
pub(crate) fn body(text: &str) -> String {
    let lines = text
        .trim()
        .lines()
        .map(|line| sanitize::text(line.trim_end()));
    lines.collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use crate::shared::calendar::{Calendar, list::CalendarListOutput};

    #[test]
    fn calendars_print_as_records() {
        let output = CalendarListOutput {
            calendars: vec![
                Calendar {
                    id: "personal".into(),
                    name: "Personal".into(),
                    description: Some("Family\nand friends\x1b[2J".into()),
                    color: Some("#ff0000".into()),
                    default: true,
                },
                Calendar {
                    id: "work".into(),
                    name: "Work".into(),
                    ..Default::default()
                },
            ],
        };

        let expected = "\
personal (default)
Name:   Personal
Color:  #ff0000

    Family
    and friends\u{fffd}[2J

work
Name:  Work
";
        assert_eq!(output.to_string(), expected);
    }
}
