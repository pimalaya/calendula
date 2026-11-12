//! # Account capability
//!
//! The `account capability` command, listing what each backend of an
//! account serves of the unified API, from the capability tables, without
//! connecting.

use std::{fmt, path::PathBuf};

use anyhow::{Result, bail};
use clap::Parser;
use pimalaya_cli::{columns::Columns, printer::Printer};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    backend::Backend,
    capability::{self, Capability},
    cli::resolve_account,
};

/// List what the account's backends serve of the unified API.
///
/// Prints, for each backend the account configures that `--backend`
/// allows, in the order `auto` tries them (the first being the one the
/// unified commands run on), every capability with its level (`full`,
/// `partial`, `server` when the server decides, `none`) and what the
/// level leaves out. Nothing is reached: this reads the configuration
/// only.
///
/// JSON output: `{"account", "backends": [{"backend", "capabilities":
/// [{"name", "level", "detail"}]}]}`.
#[derive(Debug, Parser)]
pub struct AccountCapabilityCommand;

impl AccountCapabilityCommand {
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
        backend: Backend,
    ) -> Result<()> {
        let (_, name, account_config) = resolve_account(config_paths, account_name)?;

        let backends: Vec<BackendCapabilities> = account_config
            .backends()
            .into_iter()
            .filter(|configured| backend == Backend::Auto || backend.to_string() == *configured)
            .map(|configured| BackendCapabilities {
                backend: configured,
                capabilities: Capability::ALL
                    .into_iter()
                    .map(|capability| {
                        let support = capability::support(configured, capability);
                        AccountCapability {
                            name: capability.name(),
                            level: support.level(),
                            detail: support.detail(),
                        }
                    })
                    .collect(),
            })
            .collect();

        if backends.is_empty() {
            bail!("No backend matching `{backend}` is configured for this account");
        }

        printer.out(AccountCapabilityOutput {
            account: name,
            backends,
        })
    }
}

/// The `account capability` output.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccountCapabilityOutput {
    /// The name of the account.
    pub account: String,
    /// One entry per backend, in the order `auto` tries them.
    pub backends: Vec<BackendCapabilities>,
}

/// What one backend of the account serves.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BackendCapabilities {
    /// The backend name, as `--backend` spells it.
    pub backend: &'static str,
    /// Each capability of the unified API, in a fixed order.
    pub capabilities: Vec<AccountCapability>,
}

/// One capability, as a backend serves it.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccountCapability {
    /// The capability's name, pimdir STORAGE Annex B's without the domain
    /// word, as `item.update.if-match`.
    pub name: &'static str,
    /// `full`, `partial`, `server` or `none`.
    pub level: &'static str,
    /// What the level leaves out, `null` for a full one.
    pub detail: Option<&'static str>,
}

impl fmt::Display for AccountCapabilityOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut columns = Columns::new(["CAPABILITY", "BACKEND", "LEVEL", "DETAIL"]);

        for backend in &self.backends {
            for capability in &backend.capabilities {
                columns.push([
                    capability.name,
                    backend.backend,
                    capability.level,
                    capability.detail.unwrap_or_default(),
                ]);
            }
        }

        write!(f, "{columns}")
    }
}
