//! # Calendar delete
//!
//! The `calendar delete` command, removing a calendar and everything in
//! it.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    capability::Capability,
    shared::{
        client::CalendarClient,
        receipt::{Write, WriteReceipt},
    },
};

/// Delete the given calendar and every item it contains.
///
/// JSON output: `{"id", "queued"}`.
#[derive(Debug, Parser)]
pub struct CalendarDeleteCommand {
    /// Calendar to delete, by its id.
    ///
    /// Mandatory: unlike the other unified commands, deletion never falls
    /// back to the `calendar.default` config.
    #[arg(value_name = "CALENDAR", required = true)]
    pub id: Option<String>,

    /// The calendar as a flag, the former spelling.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR")]
    #[arg(hide = true, conflicts_with = "id")]
    pub flag: Option<String>,
}

impl CalendarDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, client: &mut CalendarClient) -> Result<()> {
        client.require(&[Capability::CalendarDelete])?;

        // NOTE: clap waives the required positional when the conflicting
        // flag is present, so exactly one of the two holds the id.
        let id = self.id.or(self.flag).unwrap_or_default();
        client.delete_calendar(&id)?;

        printer.out(WriteReceipt {
            id,
            queued: client.queues_writes(),
            what: "Calendar",
            write: Write::Delete,
        })
    }
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser, error::ErrorKind};

    use super::CalendarDeleteCommand;

    fn parse(args: &[&str]) -> Result<Option<String>, ErrorKind> {
        CalendarDeleteCommand::try_parse_from([&["delete"], args].concat())
            .map(|cmd| cmd.id.or(cmd.flag))
            .map_err(|err| err.kind())
    }

    /// The calendar is a required positional, the former `-k` a hidden
    /// spelling of it, and the two together a conflict.
    #[test]
    fn the_calendar_is_a_positional_and_k_its_hidden_spelling() {
        assert_eq!(parse(&["work"]), Ok(Some(String::from("work"))));
        assert_eq!(parse(&["-k", "work"]), Ok(Some(String::from("work"))));
        assert_eq!(
            parse(&["--calendar", "work"]),
            Ok(Some(String::from("work")))
        );
        assert_eq!(
            parse(&["work", "-k", "home"]),
            Err(ErrorKind::ArgumentConflict)
        );
        assert_eq!(parse(&[]), Err(ErrorKind::MissingRequiredArgument));

        let usage = CalendarDeleteCommand::command().render_usage().to_string();
        assert_eq!(usage, "Usage: calendula <CALENDAR>");
        let help = CalendarDeleteCommand::command()
            .render_long_help()
            .to_string();
        assert!(!help.contains("-k"), "{help}");
    }
}
