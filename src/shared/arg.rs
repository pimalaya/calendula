use clap::Parser;

/// Shared `-k/--calendar` argument naming the calendar a shared-API
/// command operates on. Resolve the id through the account (the flag
/// wins, otherwise `calendar.default`, otherwise bail).
#[derive(Debug, Parser)]
pub struct CalendarIdArg {
    /// Calendar the command operates on. Falls back to the
    /// `calendar.default` config when omitted, otherwise the command
    /// bails.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR-ID")]
    pub id: Option<String>,
}
