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
        arg::CalendarIdArg,
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
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// New display name.
    #[arg(short, long, value_name = "NAME")]
    pub name: Option<String>,

    /// New description. Pass an empty string to clear.
    #[arg(short, long, value_name = "TEXT")]
    pub description: Option<String>,

    /// New hex color (`#RRGGBB`). Pass an empty string to clear.
    #[arg(long, value_name = "HEX")]
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

        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        client.update_calendar(&calendar_id, patch)?;

        printer.out(WriteReceipt {
            id: calendar_id,
            queued: client.queues_writes(),
            what: "Calendar",
            write: Write::Update,
        })
    }
}
