//! # Calendar selector
//!
//! The `-k/--calendar` argument every shared-API command carries.

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
