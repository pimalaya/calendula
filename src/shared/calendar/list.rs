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

use crate::{
    capability::Capability,
    error::{self, ItemError, ItemErrors},
    shared::{calendar::Calendar, client::CalendarClient},
};

/// List the calendars of the active account.
///
/// Every backend serves this the same way; use the protocol-specific listings
/// when you need what only one backend exposes.
///
/// JSON output: `{"calendars": [{"id", "name", "description", "color",
/// "default"}], "errors": [{"id", "code", "error", "sources"}]}`, `errors`
/// naming each calendar the backend could not read, as `Error:` lines at
/// the end of the text, the command then exiting 3.
#[derive(Debug, Parser)]
pub struct CalendarListCommand;

impl CalendarListCommand {
    pub fn execute(self, printer: &mut impl Printer, client: &mut CalendarClient) -> Result<()> {
        client.require(&[Capability::CalendarList])?;

        // NOTE: no backend names a calendar it could not read yet: each
        // reads its calendars whole or fails.
        let output = CalendarListOutput {
            calendars: client.list_calendars()?,
            errors: Vec::new(),
        };

        printer.out(&output)?;
        error::partial_when(&output.errors)
    }
}

/// The calendar listing.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CalendarListOutput {
    /// The calendars of the account.
    pub calendars: Vec<Calendar>,
    /// The error of each calendar the backend could not read, in the order
    /// met.
    pub errors: Vec<ItemError>,
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

        write!(f, "{records}")?;
        ItemErrors::new(&self.errors).write_after(f)
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
    use anyhow::anyhow;

    use crate::{
        error,
        shared::calendar::{Calendar, list::CalendarListOutput},
    };

    #[test]
    fn calendars_print_as_records() {
        let mut output = CalendarListOutput {
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
            errors: Vec::new(),
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

        // NOTE: the errors print after the records and one blank line.
        output.calendars.truncate(1);
        output.calendars[0].description = None;
        let err = anyhow!("Cannot read the metadata of calendar `old`");
        output.errors.push(error::unreadable(Some("old"), err));

        let expected = "\
personal (default)
Name:   Personal
Color:  #ff0000

Error:  old  item-unreadable  Cannot read the metadata of calendar `old`
";
        assert_eq!(output.to_string(), expected);
    }
}
