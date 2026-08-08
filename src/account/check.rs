//! The `account check` command, and the account test the wizard reuses.

use std::{fmt, path::PathBuf};

use anyhow::{Result, anyhow, bail};
use clap::Parser;
use pimalaya_cli::printer::Printer;
use pimalaya_config::toml::TomlConfig;
use serde::Serialize;

use crate::{
    backend::Backend,
    config::{AccountConfig, Config},
};

/// Validate the account configuration.
///
/// Loads the configuration, picks the active account, then exercises
/// every backend the `--backend` flag allows: a local backend is
/// checked against the filesystem, CalDAV by opening the connection and
/// resolving the calendar home-set.
///
/// JSON output: `{"account", "backends": [{"backend", "ok", "error"}]}`.
#[derive(Debug, Parser)]
pub struct AccountCheckCommand;

impl AccountCheckCommand {
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
        backend: Backend,
    ) -> Result<()> {
        let mut config = match Config::from_paths_or_default(config_paths)? {
            Some(config) => config,
            None => bail!("No configuration found. Run `calendula` to generate one."),
        };

        let (name, account_config) = config
            .take_account(account_name)?
            .ok_or_else(|| anyhow!("Cannot find account"))?;

        let backends = check_account(&account_config, backend);

        if backends.is_empty() {
            bail!("No backend matching `{backend}` is configured for this account");
        }

        printer.out(CheckReport {
            account: name,
            backends,
        })
    }
}

/// Exercises every backend of `account_config` that `backend` allows,
/// one report row each. Shared with the wizard, which tests the account
/// it just built before printing it.
pub fn check_account(
    #[allow(unused_variables)] account_config: &AccountConfig,
    #[allow(unused_variables)] backend: Backend,
) -> Vec<BackendCheck> {
    let mut checks = Vec::new();

    #[cfg(feature = "vdir")]
    if backend.allows_vdir()
        && let Some(config) = account_config.vdir.clone()
    {
        checks.push(BackendCheck::from("vdir", check_vdir(config)));
    }

    #[cfg(feature = "pimdir")]
    if backend.allows_pimdir()
        && let Some(config) = account_config.pimdir.clone()
    {
        checks.push(BackendCheck::from("pimdir", check_pimdir(config)));
    }

    #[cfg(feature = "caldav")]
    if backend.allows_caldav()
        && let Some(config) = account_config.caldav.clone()
    {
        checks.push(BackendCheck::from("caldav", check_caldav(config)));
    }

    checks
}

/// Whether every checked backend answered.
pub fn all_ok(checks: &[BackendCheck]) -> bool {
    !checks.is_empty() && checks.iter().all(|check| check.ok)
}

/// The vdir home has to be a directory that exists: everything else is
/// a per-collection concern a command reports on its own.
#[cfg(feature = "vdir")]
fn check_vdir(config: crate::config::VdirConfig) -> Result<()> {
    use std::path::PathBuf;

    let home = shellexpand::full(&config.home_dir.to_string_lossy())
        .map(|home| PathBuf::from(home.into_owned()))
        .unwrap_or_else(|_| config.home_dir.clone());

    if !home.is_dir() {
        bail!(
            "vdir home `{}` does not exist or is not a directory",
            home.display()
        );
    }

    Ok(())
}

/// Opening the store is the check: it creates the index when absent and
/// fails loudly when the path is not a usable store.
#[cfg(feature = "pimdir")]
fn check_pimdir(config: crate::config::PimdirConfig) -> Result<()> {
    crate::pimdir::client::PimdirClient::new(config)?;
    Ok(())
}

/// Connecting and resolving the calendar home-set exercises DNS,
/// TLS, authentication and the discovery chain in one go.
#[cfg(feature = "caldav")]
fn check_caldav(config: crate::config::CaldavConfig) -> Result<()> {
    crate::caldav::client::connect_and_resolve(&config)?;
    Ok(())
}

/// What `account check` reports.
#[derive(Clone, Debug, Serialize)]
pub struct CheckReport {
    /// The account that was checked.
    pub account: String,
    /// One row per checked backend.
    pub backends: Vec<BackendCheck>,
}

/// One backend's verdict.
#[derive(Clone, Debug, Serialize)]
pub struct BackendCheck {
    /// The backend name, as `--backend` spells it.
    pub backend: &'static str,
    /// Whether the backend answered.
    pub ok: bool,
    /// Why it did not, when it did not.
    pub error: Option<String>,
}

impl BackendCheck {
    fn from(backend: &'static str, result: Result<()>) -> Self {
        match result {
            Ok(()) => Self {
                backend,
                ok: true,
                error: None,
            },
            Err(err) => Self {
                backend,
                ok: false,
                error: Some(format!("{err:#}")),
            },
        }
    }
}

impl fmt::Display for CheckReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Account: {}", self.account)?;

        for check in &self.backends {
            match &check.error {
                None => writeln!(f, "  {}: OK", check.backend)?,
                Some(err) => writeln!(f, "  {}: FAIL ({err})", check.backend)?,
            }
        }

        Ok(())
    }
}
