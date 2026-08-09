use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::gcal::{
    acl::GcalAclCommand, calendars::GcalCalendarListCommand, client::GcalClient,
    colors::GcalColorsCommand, free_busy::GcalFreeBusyCommand, instances::GcalInstancesCommand,
    move_event::GcalMoveCommand, quick_add::GcalQuickAddCommand, settings::GcalSettingsCommand,
};

/// Google Calendar CLI.
///
/// Direct access to the half of the Calendar API iCalendar cannot
/// express: sharing, availability, recurrence expansion, server-side
/// parsing and the palettes. Everything a calendar and its items have
/// in common with the other backends stays in the shared API.
///
/// Push channels are not exposed: a channel delivers to an HTTPS
/// endpoint the caller must host, which a CLI has not. Neither are
/// `calendars.clear` and `transferOwnership`, both irreversible.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum GcalCommand {
    #[command(visible_alias = "ls", alias = "calendar")]
    Calendars(GcalCalendarListCommand),
    #[command(subcommand)]
    Acl(GcalAclCommand),
    FreeBusy(GcalFreeBusyCommand),
    Instances(GcalInstancesCommand),
    Move(GcalMoveCommand),
    QuickAdd(GcalQuickAddCommand),
    Colors(GcalColorsCommand),
    Settings(GcalSettingsCommand),
}

impl GcalCommand {
    pub fn execute(self, printer: &mut impl Printer, client: GcalClient) -> Result<()> {
        match self {
            Self::Calendars(cmd) => cmd.execute(printer, client),
            Self::Acl(cmd) => cmd.execute(printer, client),
            Self::FreeBusy(cmd) => cmd.execute(printer, client),
            Self::Instances(cmd) => cmd.execute(printer, client),
            Self::Move(cmd) => cmd.execute(printer, client),
            Self::QuickAdd(cmd) => cmd.execute(printer, client),
            Self::Colors(cmd) => cmd.execute(printer, client),
            Self::Settings(cmd) => cmd.execute(printer, client),
        }
    }
}
