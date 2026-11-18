//! # Google Calendar CLI
//!
//! The `calendula gcal` command tree, dispatching each protocol-specific
//! subcommand over a connected [`GcalClient`].

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::gcal::{
    acl::GcalAclCommand,
    calendars::GcalCalendarListCommand,
    client::GcalClient,
    colors::GcalColorsCommand,
    events::{
        GcalEventsCommand, instances::GcalInstancesCommand, move_event::GcalMoveCommand,
        quick_add::GcalQuickAddCommand,
    },
    free_busy::GcalFreeBusyCommand,
    request::GcalRequestCommand,
    settings::GcalSettingsCommand,
};

/// Google Calendar-specific API.
///
/// Direct access to the half of the Calendar API iCalendar cannot express:
/// sharing, availability, recurrence expansion, server-side parsing and the
/// palettes, and a raw request for the endpoints no command covers.
/// Everything a calendar and its items have in common with the other
/// backends stays in the unified API. Each command is named after the
/// API's resource, plural included.
///
/// Push channels are not exposed: a channel delivers to an HTTPS endpoint
/// the caller must host, which a CLI has not. Neither are `calendars.clear`
/// and `transferOwnership`, both irreversible.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum GcalCommand {
    #[command(visible_alias = "ls", alias = "calendar")]
    Calendars(GcalCalendarListCommand),
    #[command(subcommand)]
    Acl(GcalAclCommand),
    FreeBusy(GcalFreeBusyCommand),
    #[command(subcommand, alias = "event")]
    Events(GcalEventsCommand),
    #[command(alias = "color")]
    Colors(GcalColorsCommand),
    #[command(alias = "setting")]
    Settings(GcalSettingsCommand),
    Request(GcalRequestCommand),

    // NOTE: the former names, run as the `events` commands they became.
    #[command(hide = true, alias = "instance")]
    Instances(GcalInstancesCommand),
    #[command(hide = true)]
    Move(GcalMoveCommand),
    #[command(hide = true)]
    QuickAdd(GcalQuickAddCommand),
}

impl GcalCommand {
    pub fn execute(self, printer: &mut impl Printer, client: GcalClient) -> Result<()> {
        let result = match self {
            Self::Calendars(cmd) => cmd.execute(printer, client),
            Self::Acl(cmd) => cmd.execute(printer, client),
            Self::FreeBusy(cmd) => cmd.execute(printer, client),
            Self::Events(cmd) => cmd.execute(printer, client),
            Self::Colors(cmd) => cmd.execute(printer, client),
            Self::Settings(cmd) => cmd.execute(printer, client),
            Self::Request(cmd) => cmd.execute(printer, client),
            Self::Instances(cmd) => cmd.execute(printer, client),
            Self::Move(cmd) => cmd.execute(printer, client),
            Self::QuickAdd(cmd) => cmd.execute(printer, client),
        };

        crate::shared::refusal::print(printer, &[crate::gcal::client::refusal], result)
    }
}
