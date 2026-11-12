//! # Account check
//!
//! The `account check` command, testing each backend of an account.

use std::{fmt, path::PathBuf};

use anyhow::{Result, anyhow, bail};
use clap::Parser;
use pimalaya_cli::printer::Printer;
#[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
use pimalaya_config::secret::SecretResolver;
use pimalaya_config::toml::TomlConfig;
use schemars::JsonSchema;
use serde::Serialize;

use crate::{backend::Backend, cli::load_config, config::AccountConfig};

/// Validate the account configuration.
///
/// Loads the configuration, picks the active account, then exercises
/// every backend the `--backend` flag allows: a local backend is
/// checked against the filesystem, CalDAV by opening the connection and
/// resolving the calendar home-set, gcal by listing the calendars.
///
/// A backend that fails fails the command, with one error naming each
/// failed backend and why.
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
        let mut config = load_config(config_paths)?;

        let (name, account_config) = config
            .take_account(account_name)?
            .ok_or_else(|| anyhow!("Cannot find account"))?;

        let backends = check_account(&account_config, backend);

        if backends.is_empty() {
            bail!("No backend matching `{backend}` is configured for this account");
        }

        let failed: Vec<String> = backends
            .iter()
            .filter_map(|check| {
                let err = check.error.as_ref()?;
                Some(format!("{}: {err}", check.backend))
            })
            .collect();

        if !failed.is_empty() {
            bail!("Account `{name}` failed its check: {}", failed.join("; "));
        }

        printer.out(AccountCheckOutput {
            account: name,
            backends,
        })
    }
}

/// Exercises every backend of `account_config` that `backend` allows,
/// one report row each.
///
/// The credentials go through one resolver for the whole account, so an
/// account whose CalDAV and gcal blocks name the same command unlocks a
/// `pass` or `gpg` entry once rather than once per backend. It is
/// dropped with the check, holding plaintext while it lives.
fn check_account(
    #[allow(unused_variables)] account_config: &AccountConfig,
    #[allow(unused_variables)] backend: Backend,
) -> Vec<BackendCheck> {
    let mut checks = Vec::new();

    #[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
    let mut resolver = SecretResolver::new();

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
        checks.push(BackendCheck::from(
            "caldav",
            check_caldav(config, &mut resolver),
        ));
    }

    #[cfg(feature = "msgraph")]
    if backend.allows_msgraph()
        && let Some(config) = account_config.msgraph.clone()
    {
        checks.push(BackendCheck::from(
            "msgraph",
            check_msgraph(config, &mut resolver),
        ));
    }

    #[cfg(feature = "gcal")]
    if backend.allows_gcal()
        && let Some(config) = account_config.gcal.clone()
    {
        checks.push(BackendCheck::from(
            "gcal",
            check_gcal(config, &mut resolver),
        ));
    }

    checks
}

/// The vdir home has to be a directory that exists: everything else is
/// a per-collection concern a command reports on its own.
///
/// The path arrives shell-expanded, `VdirConfig::home_dir` doing it at
/// deserialize.
#[cfg(feature = "vdir")]
fn check_vdir(config: crate::config::VdirConfig) -> Result<()> {
    if !config.home_dir.is_dir() {
        bail!(
            "vdir home `{}` does not exist or is not a directory",
            config.home_dir.display()
        );
    }

    Ok(())
}

/// Opens the store as a reader, which creates nothing, so a path holding
/// no store, or one this build cannot read, fails it; then probes a
/// producer, which takes and releases the staging lock and the database's
/// write lock without writing anything, so a store a write cannot reach
/// fails here rather than on the first write.
#[cfg(feature = "pimdir")]
fn check_pimdir(config: crate::config::PimdirConfig) -> Result<()> {
    use crate::pimdir::client::{PimdirClient, store_error};

    let root = config.root.clone();
    PimdirClient::new(config)?
        .producer()?
        .probe()
        .map_err(|err| {
            store_error(
                err,
                format_args!("Probe pimdir store `{}` as a producer", root.display()),
            )
        })
}

/// Connecting and resolving the calendar home-set exercises DNS,
/// TLS, authentication and the discovery chain in one go.
#[cfg(feature = "caldav")]
fn check_caldav(config: crate::config::CaldavConfig, resolver: &mut SecretResolver) -> Result<()> {
    crate::caldav::client::connect(&config, resolver)?;
    Ok(())
}

/// Listing the calendars is the check: it resolves the token secret,
/// opens the TLS connection and exercises the authorization in one go,
/// which a bare connect would not.
#[cfg(feature = "msgraph")]
fn check_msgraph(
    config: crate::config::MsgraphConfig,
    resolver: &mut SecretResolver,
) -> Result<()> {
    crate::msgraph::backend::MsgraphBackend::new(config, resolver)?.list_calendars()?;
    Ok(())
}

#[cfg(feature = "gcal")]
fn check_gcal(config: crate::config::GcalConfig, resolver: &mut SecretResolver) -> Result<()> {
    crate::gcal::backend::GcalBackend::new(config, resolver)?.list_calendars()?;
    Ok(())
}

/// What `account check` reports.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccountCheckOutput {
    /// The account that was checked.
    pub account: String,
    /// One row per checked backend.
    pub backends: Vec<BackendCheck>,
}

/// One backend's verdict.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
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

impl fmt::Display for AccountCheckOutput {
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

#[cfg(all(test, feature = "vdir"))]
mod tests {
    use core::fmt;

    use anyhow::Result;
    use pimalaya_cli::printer::Printer;
    use serde::Serialize;

    use crate::{account::check::AccountCheckCommand, backend::Backend};

    /// A printer keeping what it is handed, as text.
    #[derive(Default)]
    struct Capture(String);

    impl Printer for Capture {
        fn out<T: fmt::Display + Serialize>(&mut self, data: T) -> Result<()> {
            self.0.push_str(&data.to_string());
            Ok(())
        }

        fn event<T: std::fmt::Display + serde::Serialize>(&mut self, _event: T) -> Result<()> {
            Ok(())
        }

        fn out_bytes(&mut self, _bytes: pimalaya_cli::output::Bytes) -> Result<()> {
            Ok(())
        }

        fn is_json(&self) -> bool {
            false
        }
    }

    /// A script reads the verdict from the exit code, so a backend that
    /// fails fails the command, naming it and why.
    #[test]
    fn a_failed_backend_fails_the_check() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config.toml");
        let check = |home: &std::path::Path| {
            std::fs::write(
                &config,
                format!(
                    "[accounts.test]\ndefault = true\nvdir.home-dir = {:?}\n",
                    home.display().to_string(),
                ),
            )
            .unwrap();
            let mut printer = Capture::default();
            let result = AccountCheckCommand.execute(
                &mut printer,
                core::slice::from_ref(&config),
                None,
                Backend::Auto,
            );
            (result, printer.0)
        };

        let (result, out) = check(dir.path());
        result.unwrap();
        assert!(out.contains("vdir: OK"), "{out}");

        let (result, _) = check(&dir.path().join("missing"));
        let err = format!("{:#}", result.unwrap_err());
        assert!(err.contains("vdir"), "{err}");
        assert!(err.contains("does not exist"), "{err}");
    }
}

#[cfg(all(test, feature = "pimdir"))]
mod pimdir_tests {
    use std::fs::{self, OpenOptions};

    use io_pimdir::client::PimdirStore;
    use pimalaya_cli::error::code_of;

    use crate::{account::check::check_pimdir, config::PimdirConfig, error::ErrorCode};

    /// A collector holding the staging lock fails the check at once with
    /// `store-busy`, a store the reader opens being one a write may still
    /// not reach, and the check stages nothing into the blob tree.
    #[test]
    fn a_held_staging_lock_fails_the_check_as_busy() {
        let dir = tempfile::tempdir().unwrap();
        PimdirStore::open(dir.path())
            .unwrap()
            .ensure_collection("cal", "text/calendar")
            .unwrap();
        let collector = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(dir.path().join("objects.lock"))
            .unwrap();
        collector.try_lock().unwrap();

        let config = PimdirConfig {
            root: dir.path().to_path_buf(),
            account: None,
        };
        let err = check_pimdir(config.clone()).unwrap_err();
        assert_eq!(code_of(&err), Some(ErrorCode::StoreBusy), "{err:#}");

        drop(collector);
        check_pimdir(config).unwrap();

        let objects = dir.path().join("objects");
        let left: Vec<_> = match fs::read_dir(&objects) {
            Ok(entries) => entries.map(|entry| entry.unwrap().path()).collect(),
            Err(_) => Vec::new(),
        };
        assert!(left.is_empty(), "{left:?}");
    }
}
