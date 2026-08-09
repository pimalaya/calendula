use anyhow::Result;
use clap::Parser;
use io_gcal::v3::rest::acl::GcalAclRule;
use pimalaya_cli::printer::{Message, Printer};

use crate::gcal::{acl::GcalAclRuleArgs, client::GcalClient, render};

/// Change the role a rule grants.
///
/// The rule is addressed by the id a listing showed, and the scope is
/// restated because the API replaces the whole rule.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct GcalAclUpdateCommand {
    /// The calendar the rule applies to. Falls back to
    /// `calendar.default`.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR-ID")]
    pub calendar_id: Option<String>,

    /// The rule identifier a listing showed, `<scope type>:<value>`.
    #[arg(value_name = "RULE-ID")]
    pub rule_id: String,

    #[command(flatten)]
    pub rule: GcalAclRuleArgs,

    /// Notify the grantee by email.
    #[arg(long)]
    pub notify: bool,
}

impl GcalAclUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GcalClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar_id)?;
        let rule = GcalAclRule {
            scope: Some(self.rule.scope()?),
            role: Some(render::parse_access_role(&self.rule.role)?),
            ..Default::default()
        };

        client.acl_rule_update(&calendar_id, &self.rule_id, &rule, Some(self.notify))?;

        printer.out(Message::new("Rule successfully updated"))
    }
}
