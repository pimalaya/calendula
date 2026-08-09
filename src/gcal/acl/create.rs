use anyhow::Result;
use clap::Parser;
use io_gcal::v3::rest::acl::GcalAclRule;
use pimalaya_cli::printer::{Message, Printer};

use crate::gcal::{acl::GcalAclRuleArgs, client::GcalClient, render};

/// Grant a scope a role on a calendar.
///
/// The rule id is minted by the API from the scope, so it is reported
/// rather than asked for.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct GcalAclCreateCommand {
    /// The calendar the rule applies to. Falls back to
    /// `calendar.default`.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR-ID")]
    pub calendar_id: Option<String>,

    #[command(flatten)]
    pub rule: GcalAclRuleArgs,

    /// Notify the grantee by email.
    #[arg(long)]
    pub notify: bool,
}

impl GcalAclCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GcalClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar_id)?;
        let rule = GcalAclRule {
            scope: Some(self.rule.scope()?),
            role: Some(render::parse_access_role(&self.rule.role)?),
            ..Default::default()
        };

        let created = client
            .acl_rule_insert(&calendar_id, &rule, Some(self.notify))?
            .response;
        let id = created.id.unwrap_or_default();

        printer.out(Message::new(format!("Rule `{id}` successfully created")))
    }
}
