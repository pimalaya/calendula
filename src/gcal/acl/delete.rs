use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::gcal::client::GcalClient;

/// Revoke an access control rule.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct GcalAclDeleteCommand {
    /// The calendar the rule applies to. Falls back to
    /// `calendar.default`.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR-ID")]
    pub calendar_id: Option<String>,

    /// The rule identifier a listing showed, `<scope type>:<value>`.
    #[arg(value_name = "RULE-ID")]
    pub rule_id: String,
}

impl GcalAclDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GcalClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar_id)?;
        client.acl_rule_delete(&calendar_id, &self.rule_id)?;

        printer.out(Message::new("Rule successfully deleted"))
    }
}
