//! # Shared arguments
//!
//! Clap arguments reused across the shared API commands.

use clap::Parser;

/// The calendar a shared-API command operates on.
///
/// The id resolves through the account: the flag wins, then
/// `calendar.default`, then the command bails.
#[derive(Debug, Parser)]
pub struct CalendarIdArg {
    /// Calendar the command operates on.
    ///
    /// Falls back to the `calendar.default` config when omitted, otherwise
    /// the command bails.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR-ID")]
    pub id: Option<String>,
}

/// The composer a `build`, a `create` or an `update` refines an item in.
#[derive(Debug, Parser)]
pub struct IcalComposerArgs {
    /// Edit the item in the composer before writing it.
    ///
    /// Bails when neither `item.composer` nor `--composer` names one.
    #[arg(short, long)]
    pub interactive: bool,
    /// Command the item is edited in, overriding `item.composer`.
    ///
    /// A shell line, spawned on the path of a temporary iCalendar file it
    /// edits in place.
    #[arg(long, value_name = "COMMAND", requires = "interactive")]
    pub composer: Option<String>,
}
