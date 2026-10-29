//! # Shared arguments
//!
//! Clap arguments reused across the shared API commands.

use anyhow::{Result, bail};
use clap::{Parser, ValueEnum};

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

/// Who a gcal write mails, for an `event create`, `update` or `delete`.
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

impl SendUpdatesArg {
    /// Refuses `--send-updates` off gcal, `backend` naming the one serving
    /// the account, before anything reaches it.
    pub fn check_backend(&self, backend: &str) -> Result<()> {
        if self.send_updates.is_some() && backend != "gcal" {
            bail!(
                "--send-updates needs the gcal backend; the {backend} backend serves this account"
            );
        }

        Ok(())
    }
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
