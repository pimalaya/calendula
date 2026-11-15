//! # Calendar list
//!
//! The `calendar list` command, printing the account's calendars one line
//! each.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{listing::Listing, printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    capability::Capability,
    error::{self, ItemError},
    shared::{calendar::Calendar, client::CalendarClient},
};

/// List the calendars of the active account.
///
/// Every backend serves this the same way; use the protocol-specific listings
/// when you need what only one backend exposes.
///
/// Each line shows the id of a calendar, `default` on the one the server
/// names the default, its color, then its name, under a header naming the
/// account. A calendar the backend could not read is named by an `Error:`
/// line at the end, and the command exits 3.
///
/// JSON output: `{"calendars": [{"id", "name", "description", "color",
/// "default"}], "errors": [{"id", "code", "error", "sources"}]}`, `errors`
/// naming each calendar the backend could not read.
#[derive(Debug, Parser)]
pub struct CalendarListCommand;

impl CalendarListCommand {
    pub fn execute(self, printer: &mut impl Printer, client: &mut CalendarClient) -> Result<()> {
        client.require(&[Capability::CalendarList])?;

        // NOTE: no backend names a calendar it could not read yet: each
        // reads its calendars whole or fails.
        let output = CalendarListOutput {
            listing: Listing::new(&client.name, Some(client.backend_name())),
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
    /// The header the calendars print under.
    #[serde(skip)]
    pub listing: Listing<'static>,
    /// The calendars of the account.
    pub calendars: Vec<Calendar>,
    /// The error of each calendar the backend could not read, in the order
    /// met.
    pub errors: Vec<ItemError>,
}

impl fmt::Display for CalendarListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut listing = self.listing.clone();

        for calendar in &self.calendars {
            let default = match calendar.default {
                true => String::from("default"),
                false => String::new(),
            };

            listing.items.push([
                sanitize::text(&calendar.id).into_owned(),
                default,
                sanitize::text(calendar.color.as_deref().unwrap_or_default()).into_owned(),
                sanitize::text(&calendar.name).into_owned(),
            ]);
        }

        for error in &self.errors {
            listing.errors.push(error);
        }

        write!(f, "{listing}")
    }
}

/// A free text as a record body, each line defused.
#[cfg(any(feature = "caldav", feature = "gcal"))]
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
    use pimalaya_cli::listing::Listing;

    use crate::{
        error,
        shared::calendar::{Calendar, list::CalendarListOutput},
    };

    /// The calendars print one line each under the account, as Himalaya's
    /// mailboxes do: the id first, the name last, `-` for an empty cell,
    /// and the errors after the lines.
    #[test]
    fn calendars_print_one_line_each_under_the_account() {
        let mut output = CalendarListOutput {
            listing: Listing::new("perso", Some("vdir")),
            calendars: vec![
                Calendar {
                    id: "personal".into(),
                    name: "Personal\x1b[2J".into(),
                    description: Some("Family\nand friends".into()),
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
Account:  perso (vdir)

personal  default  #ff0000  Personal\u{fffd}[2J
work      -        -        Work
";
        assert_eq!(output.to_string(), expected);

        output.calendars.truncate(1);
        let err = anyhow!("Cannot read the metadata of calendar `old`");
        output.errors.push(error::unreadable(Some("old"), err));

        let expected = "\
Account:  perso (vdir)

personal  default  #ff0000  Personal\u{fffd}[2J

Error:  old  item-unreadable  Cannot read the metadata of calendar `old`
";
        assert_eq!(output.to_string(), expected);
    }
}
