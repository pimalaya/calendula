//! # Account capability
//!
//! The `account capability` command, listing what the backend an account
//! resolves to serves of the unified API, from the capability tables,
//! without connecting.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{
    capability::{AccountCapability, AccountCapabilityOutput},
    printer::Printer,
};

use crate::{
    backend::Backend,
    capability::{self, Capability, Support},
    cli::resolve_account,
    error,
    shared::client::resolve_backend,
};

/// What a capability reads when no backend of the account is allowed.
const NO_BACKEND: Support = Support::None("the account configures no backend `--backend` allows");

/// List what the account serves of the unified API.
///
/// Resolves the backend the unified commands run against (the first
/// configured one `--backend` allows, in the order `auto` tries them) and
/// prints, for each capability, that backend, its level (`full`,
/// `partial`, `server` when the server decides, `none`) and what the
/// level leaves out. Pass `--backend <name>` to read another backend of
/// the account. Nothing is reached: this reads the configuration only.
/// A backend this build leaves out fails it with `backend-not-built`, as
/// it fails the unified commands.
///
/// JSON output: `{"account", "capabilities": [{"name", "backend",
/// "level", "detail"}]}`, `backend` `null` when the account configures
/// none that `--backend` allows.
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
        let backend = resolve_backend(&account_config, backend)
            .map_err(|left_out| error::backend_not_built(left_out, &account_config.backends()))?
            .map(|backend| backend.to_string());

        let capabilities = Capability::ALL
            .into_iter()
            .map(|capability| {
                let support = match &backend {
                    Some(backend) => capability::support(backend, capability),
                    None => NO_BACKEND,
                };

                AccountCapability {
                    name: capability.name().to_owned(),
                    backend: backend.clone(),
                    level: support.level(),
                    detail: support.detail().map(ToOwned::to_owned),
                }
            })
            .collect();

        printer.out(AccountCapabilityOutput {
            account: name,
            capabilities,
        })
    }
}
