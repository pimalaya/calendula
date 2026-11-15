//! # Calendar update
//!
//! The `calendar update` command, patching a calendar's mutable
//! properties.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    capability::Capability,
    shared::{
        calendar::CalendarDiff,
        client::CalendarClient,
        receipt::{Write, WriteReceipt},
    },
};

/// Update a calendar's mutable properties.
///
/// Each flag updates only its own field and an omitted one is left untouched.
/// Pass an empty value to clear an optional field, as in `--description ""`.
///
/// JSON output: `{"id", "queued"}`.
#[derive(Debug, Parser)]
pub struct CalendarUpdateCommand {
    /// Calendar to update, by its id.
    ///
    /// Falls back to the `calendar.default` config when omitted, otherwise
    /// the command fails coded `calendar-default-missing`.
    #[arg(value_name = "CALENDAR")]
    pub id: Option<String>,

    /// The calendar as a flag, the former spelling.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR")]
    #[arg(hide = true, conflicts_with = "id")]
    pub flag: Option<String>,

    /// New display name.
    #[arg(short, long, value_name = "NAME")]
    pub name: Option<String>,

    /// New description. Pass an empty string to clear.
    #[arg(short, long, value_name = "TEXT")]
    pub description: Option<String>,

    /// New hex color (`#RRGGBB`). Pass an empty string to clear.
    #[arg(short = 'C', long, value_name = "HEX")]
    pub color: Option<String>,
}

impl CalendarUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, client: &mut CalendarClient) -> Result<()> {
        let mut capabilities = Vec::new();
        if self.name.is_some() {
            capabilities.push(Capability::CalendarRename);
        }
        if self.description.is_some() {
            capabilities.push(Capability::CalendarDescription);
        }
        if self.color.is_some() {
            capabilities.push(Capability::CalendarColor);
        }
        client.require(&capabilities)?;

        let patch = CalendarDiff {
            name: self.name,
            description: self
                .description
                .map(|s| if s.is_empty() { None } else { Some(s) }),
            color: self
                .color
                .map(|s| if s.is_empty() { None } else { Some(s) }),
        };

        let calendar_id = client.account.calendar_id(self.id.or(self.flag))?;
        client.update_calendar(&calendar_id, patch)?;

        printer.out(WriteReceipt {
            id: calendar_id,
            queued: client.queues_writes(),
            what: "Calendar",
            write: Write::Update,
        })
    }
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser, error::ErrorKind};

    use super::CalendarUpdateCommand;

    fn parse(args: &[&str]) -> Result<Option<String>, ErrorKind> {
        CalendarUpdateCommand::try_parse_from([&["update"], args].concat())
            .map(|cmd| cmd.id.or(cmd.flag))
            .map_err(|err| err.kind())
    }

    /// The calendar is an optional positional, falling back to the
    /// default, the former `-k` a hidden spelling of it, and the two
    /// together a conflict.
    #[test]
    fn the_calendar_is_a_positional_and_k_its_hidden_spelling() {
        assert_eq!(parse(&["work", "-n", "Job"]), Ok(Some("work".into())));
        assert_eq!(parse(&["-k", "work", "-n", "Job"]), Ok(Some("work".into())));
        assert_eq!(parse(&["-n", "Job"]), Ok(None));
        assert_eq!(
            parse(&["work", "-k", "home"]),
            Err(ErrorKind::ArgumentConflict)
        );

        let usage = CalendarUpdateCommand::command().render_usage().to_string();
        assert_eq!(usage, "Usage: calendula [OPTIONS] [CALENDAR]");
        let help = CalendarUpdateCommand::command()
            .render_long_help()
            .to_string();
        assert!(!help.contains("-k,"), "{help}");
    }
}
