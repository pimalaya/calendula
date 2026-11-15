//! # ACL update
//!
//! The `calendula gcal acl update` command, changing the role an existing
//! rule grants.

use anyhow::Result;
use clap::Parser;
use io_gcal::v3::rest::acl::{GcalAclRule, GcalAclScope, GcalAclScopeType};
use pimalaya_cli::{
    error::{CodedError, CommonCode},
    printer::Printer,
};

use crate::{
    gcal::{client::GcalClient, render},
    shared::receipt::IdReceipt,
};

/// Change the role a rule grants.
///
/// The rule is addressed by the id a listing showed, `<scope type>:<value>`
/// (`default` for everyone), which names its scope: the API replaces the
/// whole rule, so the scope is read from the id and restated.
///
/// JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct GcalAclUpdateCommand {
    /// The calendar the rule applies to. Falls back to
    /// `calendar.default`.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR-ID")]
    pub calendar_id: Option<String>,

    /// The rule identifier a listing showed, `<scope type>:<value>`.
    #[arg(value_name = "RULE-ID")]
    pub rule_id: String,

    /// The scope type, read from the rule id when omitted; one disagreeing
    /// with it is refused.
    #[arg(long, value_name = "TYPE")]
    pub scope: Option<String>,

    /// The scope value, read from the rule id when omitted; one disagreeing
    /// with it is refused.
    #[arg(long, value_name = "VALUE")]
    pub value: Option<String>,

    /// What the rule grants: `none`, `freeBusyReader`, `reader`,
    /// `writerWithoutPrivateAccess`, `writer` or `owner`.
    #[arg(long, value_name = "ROLE")]
    pub role: String,

    /// Notify the grantee by email.
    #[arg(long)]
    pub notify: bool,
}

impl GcalAclUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GcalClient) -> Result<()> {
        let scope = rule_scope(&self.rule_id, self.scope.as_deref(), self.value.as_deref())?;
        let role = render::parse_access_role(&self.role)?;
        let calendar_id = client.account.calendar_id(self.calendar_id)?;
        let rule = GcalAclRule {
            scope: Some(scope),
            role: Some(role),
            ..Default::default()
        };

        client.acl_rule_update(&calendar_id, &self.rule_id, &rule, Some(self.notify))?;

        printer.out(IdReceipt {
            id: self.rule_id,
            done: "Rule updated",
        })
    }
}

/// The scope the rule `rule_id` names, `<scope type>:<value>` or
/// `default`, checked against the `scope` and `value` a caller restated.
///
/// The API answers a missing value with a 400 and a disagreeing one with
/// a 403, which reads as a matter of rights, so both are settled here,
/// a disagreement refused `usage-invalid`.
fn rule_scope(rule_id: &str, scope: Option<&str>, value: Option<&str>) -> Result<GcalAclScope> {
    let (named, named_value) = match rule_id.split_once(':') {
        Some((scope, value)) => (scope, Some(value)),
        None => (rule_id, None),
    };

    let scope_type = render::parse_scope_type(named)
        .map_err(|_| usage(format!("Rule id `{rule_id}` is not `<scope type>:<value>`")))?;

    let named_value = match (&scope_type, named_value) {
        (GcalAclScopeType::Default, None) => None,
        (GcalAclScopeType::Default, Some(_)) | (_, None) | (_, Some("")) => {
            return Err(usage(format!(
                "Rule id `{rule_id}` is not `<scope type>:<value>`"
            )));
        }
        (_, Some(value)) => Some(value.to_owned()),
    };

    if let Some(scope) = scope
        && render::parse_scope_type(scope).ok().as_ref() != Some(&scope_type)
    {
        return Err(usage(format!(
            "--scope {scope} disagrees with rule `{rule_id}`; leave it out"
        )));
    }

    let agrees = |given: &str| {
        named_value
            .as_deref()
            .is_some_and(|named| named.eq_ignore_ascii_case(given))
    };

    if let Some(value) = value
        && !agrees(value)
    {
        return Err(usage(format!(
            "--value {value} disagrees with rule `{rule_id}`; leave it out"
        )));
    }

    Ok(GcalAclScope {
        scope_type: Some(scope_type),
        value: named_value,
    })
}

/// A refusal of the command line, coded `usage-invalid`, exit 2.
fn usage(message: String) -> anyhow::Error {
    CodedError::new(CommonCode::UsageInvalid, message).into()
}

#[cfg(test)]
mod tests {
    use io_gcal::v3::rest::acl::GcalAclScopeType;
    use pimalaya_cli::error::{CommonCode, code_of};

    use super::rule_scope;

    /// The scope and its value come from the rule id, a restated one
    /// agreeing with it, whatever its case.
    #[test]
    fn the_scope_is_read_from_the_rule_id() {
        let scope = rule_scope("user:a@b.org", None, None).unwrap();
        assert_eq!(scope.scope_type, Some(GcalAclScopeType::User));
        assert_eq!(scope.value.as_deref(), Some("a@b.org"));

        let scope = rule_scope("group:team@b.org", Some("group"), Some("Team@B.org")).unwrap();
        assert_eq!(scope.scope_type, Some(GcalAclScopeType::Group));
        assert_eq!(scope.value.as_deref(), Some("team@b.org"));

        let scope = rule_scope("default", None, None).unwrap();
        assert_eq!(scope.scope_type, Some(GcalAclScopeType::Default));
        assert_eq!(scope.value, None);
    }

    /// A value or a scope disagreeing with the rule id, or an id naming no
    /// scope, is a usage error, never sent.
    #[test]
    fn a_disagreeing_scope_or_value_is_a_usage_error() {
        let cases = [
            ("user:a@b.org", None, Some("c@d.org")),
            ("user:a@b.org", Some("group"), None),
            ("default", None, Some("a@b.org")),
            ("nonsense", None, None),
            ("user:", None, None),
            ("user", None, None),
        ];

        for (id, scope, value) in cases {
            let err = rule_scope(id, scope, value).unwrap_err();
            let code = code_of::<CommonCode>(&err);
            assert_eq!(code, Some(CommonCode::UsageInvalid), "{id}: {err}");
        }
    }
}
