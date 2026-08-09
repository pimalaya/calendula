//! The `gcal acl` subcommands: the sharing rules of a calendar.
//!
//! A rule grants one scope (a single user, a group, a whole domain, or
//! everyone) one role. Its id is `<scope type>:<scope value>`, minted
//! by the API, so the write verbs take the scope and the role as flags
//! and derive the id rather than asking a caller to spell a composite
//! key by hand.

pub mod create;
pub mod delete;
pub mod list;
pub mod update;

use anyhow::Result;
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
    /// Omitted for the `default` scope, which designates everyone.
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

        Ok(GcalAclScope {
            scope_type: Some(scope_type),
            // NOTE: the default scope is everyone, and the API rejects
            // a value alongside it.
            value: match scope_type {
                GcalAclScopeType::Default => None,
                _ => self.value.clone(),
            },
        })
    }
}
