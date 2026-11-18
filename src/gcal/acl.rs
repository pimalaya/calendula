//! # ACL
//!
//! The `calendula gcal acl` subcommands, over the sharing rules of a
//! calendar.
//!
//! A rule grants one scope (a user, a group, a domain, or everyone) one
//! role. Its id is `<scope type>:<scope value>`, minted by the API, so the
//! write verbs take scope and role as flags and derive the id rather than
//! asking a caller to spell a composite key by hand.

pub mod create;
pub mod delete;
pub mod list;
pub mod update;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use io_gcal::v3::rest::acl::{GcalAclScope, GcalAclScopeType};
use pimalaya_cli::printer::Printer;

use crate::gcal::{
    acl::{
        create::GcalAclCreateCommand, delete::GcalAclDeleteCommand, list::GcalAclListCommand,
        update::GcalAclUpdateCommand,
    },
    client::GcalClient,
    render,
};

/// Manage the access control rules of a calendar.
#[derive(Debug, Subcommand)]
pub enum GcalAclCommand {
    #[command(visible_alias = "ls")]
    List(GcalAclListCommand),
    Create(GcalAclCreateCommand),
    Update(GcalAclUpdateCommand),
    Delete(GcalAclDeleteCommand),
}

impl GcalAclCommand {
    pub fn execute(self, printer: &mut impl Printer, client: GcalClient) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.execute(printer, client),
            Self::Create(cmd) => cmd.execute(printer, client),
            Self::Update(cmd) => cmd.execute(printer, client),
            Self::Delete(cmd) => cmd.execute(printer, client),
        }
    }
}

/// The scope and role flags the write verbs share.
#[derive(Debug, Parser)]
pub struct GcalAclRuleArgs {
    /// Who the rule grants access to: `user`, `group`, `domain` or
    /// `default` (everyone).
    #[arg(long, value_name = "TYPE", default_value = "user")]
    pub scope: String,

    /// The address of the user or group, or the name of the domain.
    /// Omitted for the `default` scope, which is everyone.
    #[arg(long, value_name = "VALUE")]
    pub value: Option<String>,

    /// What the rule grants: `none`, `freeBusyReader`, `reader`,
    /// `writerWithoutPrivateAccess`, `writer` or `owner`.
    #[arg(long, value_name = "ROLE")]
    pub role: String,
}

impl GcalAclRuleArgs {
    /// Resolves the flags into the scope the API takes.
    pub fn scope(&self) -> Result<GcalAclScope> {
        let scope_type = render::parse_scope_type(&self.scope)?;

        let value = match (&scope_type, &self.value) {
            (GcalAclScopeType::Default, None) => None,
            (GcalAclScopeType::Default, Some(value)) => bail!(
                "--scope default is everyone and takes no --value; give --scope user, group or \
                 domain to grant `{value}`"
            ),
            (_, Some(value)) => Some(value.clone()),
            (_, None) => bail!(
                "--scope {} needs --value, the address or the domain it grants",
                self.scope
            ),
        };

        Ok(GcalAclScope {
            scope_type: Some(scope_type),
            value,
        })
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::GcalAclRuleArgs;

    #[derive(Parser)]
    struct Wrap {
        #[command(flatten)]
        rule: GcalAclRuleArgs,
    }

    fn scope(args: &[&str]) -> anyhow::Result<io_gcal::v3::rest::acl::GcalAclScope> {
        let args = [&["acl"], args].concat();
        Wrap::try_parse_from(args).unwrap().rule.scope()
    }

    /// CAL-19: a scope needing a value is refused without one, and the
    /// default scope, everyone, refuses one rather than dropping it.
    #[test]
    fn a_scope_and_its_value_are_settled_before_any_request() {
        let err = scope(&["--role", "reader"]).unwrap_err();
        assert!(err.to_string().contains("needs --value"), "{err}");

        let err = scope(&[
            "--role", "reader", "--scope", "default", "--value", "a@b.org",
        ])
        .unwrap_err();
        assert!(err.to_string().contains("takes no --value"), "{err}");

        let user = scope(&["--role", "reader", "--value", "a@b.org"]).unwrap();
        assert_eq!(user.value.as_deref(), Some("a@b.org"));
        let everyone = scope(&["--role", "reader", "--scope", "default"]).unwrap();
        assert_eq!(everyone.value, None);
    }
}
