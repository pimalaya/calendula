//! # Events
//!
//! The `calendula gcal events` subcommands, over the methods of the
//! Calendar API's Events resource the unified API has no room for:
//! `events.instances`, `events.move` and `events.quickAdd`.

pub mod instances;
pub mod move_event;
pub mod quick_add;

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::gcal::{
    client::GcalClient,
    events::{
        instances::GcalInstancesCommand, move_event::GcalMoveCommand,
        quick_add::GcalQuickAddCommand,
    },
};

/// Run the Events methods the unified API cannot express.
///
/// Named after the Calendar API's Events resource: the occurrences of a
/// series, an event moved to another calendar, and an event parsed from a
/// sentence. Reading and writing an event stays in the unified `event`
/// API.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum GcalEventsCommand {
    #[command(alias = "instance")]
    Instances(GcalInstancesCommand),
    Move(GcalMoveCommand),
    QuickAdd(GcalQuickAddCommand),
}

impl GcalEventsCommand {
    pub fn execute(self, printer: &mut impl Printer, client: GcalClient) -> Result<()> {
        match self {
            Self::Instances(cmd) => cmd.execute(printer, client),
            Self::Move(cmd) => cmd.execute(printer, client),
            Self::QuickAdd(cmd) => cmd.execute(printer, client),
        }
    }
}
