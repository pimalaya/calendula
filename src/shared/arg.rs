//! # Shared arguments
//!
//! Clap arguments reused across the unified API commands.

use clap::{Parser, ValueEnum};

/// The calendar a shared-API command operates on.
///
/// The id resolves through the account: the flag wins, then
/// `calendar.default`, else the command fails coded
/// `calendar-default-missing`.
#[derive(Debug, Parser)]
pub struct CalendarIdArg {
    /// Calendar the command operates on.
    ///
    /// Falls back to the `calendar.default` config when omitted, otherwise
    /// the command fails coded `calendar-default-missing`.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR-ID")]
    pub id: Option<String>,
}

/// The composer a `build`, an `add` or an `update` refines an event in.
#[derive(Debug, Parser)]
pub struct IcalComposerArgs {
    /// Edit the event in the composer before writing it.
    ///
    /// Bails when neither `event.composer` nor `--composer` names one.
    #[arg(short, long)]
    pub interactive: bool,
    /// Command the event is edited in, overriding `event.composer`.
    ///
    /// A shell line, spawned on the path of a temporary iCalendar file it
    /// edits in place.
    #[arg(long, value_name = "COMMAND", requires = "interactive")]
    pub composer: Option<String>,
}

/// Who a gcal write mails, for an `event add`, `update` or `delete`.
#[derive(Debug, Parser)]
pub struct SendUpdatesArg {
    /// Who Google mails about the write (`sendUpdates`): every guest,
    /// the guests outside Google Calendar, or nobody.
    ///
    /// Omitted, the write mails whom it mailed before: an update or a
    /// delete leaves it to Google, and a create invites the attendees of
    /// an event the account organizes. An event organized by someone else
    /// is imported, which mails nobody, so `all` and `external-only` are
    /// refused on it.
    ///
    /// Served by the gcal backend alone; every other backend refuses it.
    #[arg(long, value_name = "WHO")]
    pub send_updates: Option<SendUpdates>,
}

/// Who Google mails about a write.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum SendUpdates {
    /// Every guest.
    All,
    /// The guests outside Google Calendar, who keep getting iCalendar
    /// invitations.
    ExternalOnly,
    /// Nobody.
    None,
}
