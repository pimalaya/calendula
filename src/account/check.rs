//! # Account check
//!
//! The `account check` command, testing each backend of an account, as
//! pimalaya-cli's shared [`AccountCheckOutput`] prints it.

use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::Parser;
#[cfg(any(
    feature = "vdir",
    feature = "caldav",
    feature = "pimdir",
    feature = "gcal",
    feature = "msgraph"
))]
use pimalaya_cli::error::with_code;
use pimalaya_cli::{account::AccountCheckOutput, error::PartialOutcome, printer::Printer};
#[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
use pimalaya_config::secret::SecretResolver;

use crate::{backend::Backend, cli::resolve_account, config::AccountConfig, error::ErrorCode};

/// One backend's verdict, coded by calendula's vocabulary.
pub type BackendCheck = pimalaya_cli::account::BackendCheck<ErrorCode>;

/// Validate the account configuration.
///
/// Loads the configuration, picks the active account, then exercises
/// every backend the `--backend` flag allows: a local backend is
/// checked against the filesystem, CalDAV by resolving the calendar
/// home-set or, on a pinned `home`, by a PROPFIND on it, so a refused
/// credential fails either way, gcal by listing the calendars. Under
/// `auto`, a block of a backend this build leaves out fails its row with
/// `backend-not-built`, naming the cargo feature bringing it back.
///
/// The report is printed whole, each backend with its outcome and, when
/// it failed, why, its stable code under `--json`. A failed backend exits
/// the command with status 3, the report being the output rather than a
/// failure object.
///
/// JSON output: `{"account", "backends": [{"backend", "ok", "code",
/// "error"}]}`.
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
        let (_, name, account_config) = resolve_account(config_paths, account_name)?;

        let backends = check_account(&account_config, backend);

        if backends.is_empty() {
            bail!("No backend matching `{backend}` is configured for this account");
        }

        let failed = backends.iter().any(|check| !check.ok);

        printer.out(AccountCheckOutput {
            account: name,
            backends,
        })?;

        if failed {
            return Err(PartialOutcome::default().into());
        }

        Ok(())
    }
}

/// Exercises every backend of `account_config` that `backend` allows,
/// one report row each.
///
/// The credentials go through one resolver for the whole account, so an
/// account whose CalDAV and gcal blocks name the same command unlocks a
/// `pass` or `gpg` entry once rather than once per backend. It is
/// dropped with the check, holding plaintext while it lives.
fn check_account(account_config: &AccountConfig, backend: Backend) -> Vec<BackendCheck> {
    let mut checks = Vec::new();

    #[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
    let mut resolver = SecretResolver::new();

    #[cfg(feature = "vdir")]
    if backend.allows_vdir()
        && let Some(config) = account_config.vdir.clone()
    {
        checks.push(checked("vdir", check_vdir(config)));
    }

    #[cfg(feature = "pimdir")]
    if backend.allows_pimdir()
        && let Some(config) = account_config.pimdir.clone()
    {
        checks.push(checked("pimdir", check_pimdir(config)));
    }

    #[cfg(feature = "caldav")]
    if backend.allows_caldav()
        && let Some(config) = account_config.caldav.clone()
    {
        checks.push(checked("caldav", check_caldav(config, &mut resolver)));
    }

    #[cfg(feature = "msgraph")]
    if backend.allows_msgraph()
        && let Some(config) = account_config.msgraph.clone()
    {
        checks.push(checked("msgraph", check_msgraph(config, &mut resolver)));
    }

    #[cfg(feature = "gcal")]
    if backend.allows_gcal()
        && let Some(config) = account_config.gcal.clone()
    {
        checks.push(checked("gcal", check_gcal(config, &mut resolver)));
    }

    // NOTE: `auto` reports a backend this build leaves out rather than
    // skips it, as it does for a unified command.
    if backend == Backend::Auto {
        let not_built = account_config.not_built().into_iter();
        checks.extend(not_built.map(left_out));
    }

    checks
}

/// The vdir home has to be a directory that exists: everything else is
/// a per-collection concern a command reports on its own.
///
/// The path arrives shell-expanded, `VdirConfig::root` doing it at
/// deserialize.
#[cfg(feature = "vdir")]
fn check_vdir(config: crate::config::VdirConfig) -> Result<()> {
    if !config.root.is_dir() {
        bail!(
            "vdir home `{}` does not exist or is not a directory",
            config.root.display()
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
/// TLS, authentication and the discovery chain in one go. A pinned `home`
/// skips discovery, so a PROPFIND Depth 0 on it exercises the credential,
/// through the client, sent again on a fresh connection when the server
/// closed the one it was sent on.
/// A 401 to a block with no `auth` says the block configures none.
#[cfg(feature = "caldav")]
fn check_caldav(config: crate::config::CaldavConfig, resolver: &mut SecretResolver) -> Result<()> {
    use io_webdav::rfc4918::{RESOURCETYPE, WebdavMultistatus, propfind::WebdavPropfind};

    use crate::caldav::client::{connect, unauthenticated};

    let mut check = || -> Result<()> {
        let mut client = connect(&config, resolver)?;

        let Some(home) = &config.home else {
            return Ok(());
        };

        let propfind = WebdavPropfind::<WebdavMultistatus>::new(
            &client.base_url,
            client.auth(),
            &client.user_agent,
            home.path(),
            0,
            &[RESOURCETYPE],
        );
        client.run(propfind)?;

        Ok(())
    };

    check().map_err(|err| unauthenticated(err, &config.auth))
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

/// The row of `backend`, a backend this build leaves out.
fn left_out(backend: &'static str) -> BackendCheck {
    BackendCheck {
        backend,
        ok: false,
        code: Some(ErrorCode::BackendNotBuilt),
        error: Some(crate::backend::not_built(backend)),
    }
}

/// The row of `backend` that `result` checked, coded by calendula's chain
/// walk, which reads a server refusal off its typed error.
#[cfg(any(
    feature = "vdir",
    feature = "caldav",
    feature = "pimdir",
    feature = "gcal",
    feature = "msgraph"
))]
fn checked(backend: &'static str, result: Result<()>) -> BackendCheck {
    let result = result.map_err(|err| with_code(err, crate::error::code));
    BackendCheck::new(backend, result)
}

#[cfg(all(test, feature = "vdir"))]
mod tests {
    use core::fmt;

    use anyhow::Result;
    use pimalaya_cli::{error::PartialOutcome, printer::Printer};
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

    /// A failed backend still prints the report, naming it and why, and
    /// ends the command with the partial-outcome status.
    #[test]
    fn a_failed_backend_prints_the_report_and_fails_the_check() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config.toml");
        let check = |home: &std::path::Path| {
            std::fs::write(
                &config,
                format!(
                    "[accounts.test]\ndefault = true\nvdir.root = {:?}\n",
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
        assert_eq!(out, "Account:  test\nvdir:     OK\n");

        let (result, out) = check(&dir.path().join("missing"));
        let err = result.unwrap_err();
        let partial = err.downcast_ref::<PartialOutcome>();
        assert_eq!(partial.map(PartialOutcome::status), Some(3), "{err:#}");
        assert!(out.starts_with("Account:  test\nvdir:     FAIL ("), "{out}");
        assert!(out.contains("does not exist"), "{out}");
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

#[cfg(all(test, feature = "msgraph"))]
mod msgraph_tests {
    use crate::{
        account::check::checked,
        error::ErrorCode,
        msgraph::backend::scripted::{answering, refusal},
    };

    /// A backend the server refuses fails its row with the refusal's code.
    #[test]
    fn a_refused_backend_row_carries_the_refusal_code() {
        let denied = refusal("ErrorAccessDenied");
        let result = answering(&[(403, &denied)]).list_calendars().map(|_| ());
        let check = checked("msgraph", result);

        assert!(!check.ok);
        assert_eq!(check.code, Some(ErrorCode::PermissionDenied));
    }
}

#[cfg(all(test, feature = "caldav"))]
mod caldav_tests {
    use std::{
        io::{BufRead, BufReader, Read, Write},
        net::TcpListener,
        sync::mpsc::{self, Receiver},
        thread,
    };

    use pimalaya_config::secret::SecretResolver;

    use crate::{
        account::check::{BackendCheck, check_caldav, checked},
        config::CaldavConfig,
    };

    /// The multistatus of a home answering its PROPFIND.
    const HOME: &str = "<?xml version=\"1.0\"?><d:multistatus xmlns:d=\"DAV:\">\
        <d:response><d:href>/calendars/alice/</d:href><d:propstat><d:prop>\
        <d:resourcetype><d:collection/></d:resourcetype></d:prop>\
        <d:status>HTTP/1.1 200 OK</d:status></d:propstat></d:response>\
        </d:multistatus>";

    /// A local server answering every request with `status` and `body`, and
    /// the head of each request it read.
    fn answering(status: &'static str, body: &'static str) -> (String, Receiver<String>) {
        serving(status, body, false)
    }

    /// A local server closing its first connection once it read the
    /// request, unanswered, then answering as [`answering`] does.
    fn closing_first(status: &'static str, body: &'static str) -> (String, Receiver<String>) {
        serving(status, body, true)
    }

    /// A local server answering every request with `status` and `body`, the
    /// one of the first connection too unless `close_first` is set, and the
    /// head of each request it read.
    fn serving(
        status: &'static str,
        body: &'static str,
        close_first: bool,
    ) -> (String, Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let (sender, heads) = mpsc::channel();

        thread::spawn(move || {
            for (n, stream) in listener.incoming().enumerate() {
                let mut stream = BufReader::new(stream.unwrap());
                let mut head = String::new();
                let mut len = 0;

                loop {
                    let mut line = String::new();
                    if stream.read_line(&mut line).unwrap() == 0 || line == "\r\n" {
                        break;
                    }

                    let lower = line.to_ascii_lowercase();
                    if let Some(value) = lower.strip_prefix("content-length:") {
                        len = value.trim().parse().unwrap();
                    }

                    head.push_str(&line);
                }

                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: application/xml\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );

                stream.read_exact(&mut vec![0; len]).unwrap();

                if close_first && n == 0 {
                    sender.send(head).unwrap();
                    continue;
                }

                stream.get_mut().write_all(response.as_bytes()).unwrap();
                sender.send(head).unwrap();
            }
        });

        (origin, heads)
    }

    /// The row of a check over `route`, a made-up password presented.
    fn check(route: String) -> BackendCheck {
        let document = format!(
            "{route}\nauth.basic.username = \"alice\"\n\
             auth.basic.password.raw = \"made-up\""
        );
        let config: CaldavConfig = toml::from_str(&document).unwrap();

        checked("caldav", check_caldav(config, &mut SecretResolver::new()))
    }

    /// A credential the server refuses fails a pinned `home` as it fails
    /// discovery, with the same code and error, on one PROPFIND Depth 0.
    #[test]
    fn a_refused_credential_fails_the_home_route_as_the_server_route() {
        let (origin, heads) = answering("401 Unauthorized", "Unauthorized");

        let server = check(format!("server = \"{origin}/\""));
        assert!(!server.ok, "{server:?}");
        assert!(heads.recv().unwrap().starts_with("PROPFIND / "));

        let home = check(format!("home = \"{origin}/calendars/alice/\""));
        assert!(!home.ok, "{home:?}");
        assert_eq!(home.code, server.code, "{home:?}");
        assert_eq!(home.error, server.error, "{home:?}");
        assert!(home.error.unwrap().contains("HTTP 401"));

        let head = heads.recv().unwrap();
        let depth = head.to_ascii_lowercase().contains("\r\ndepth: 0\r\n");
        assert!(head.starts_with("PROPFIND /calendars/alice/ "), "{head}");
        assert!(depth, "{head}");
        assert!(heads.try_recv().is_err());
    }

    /// A 401 to a block with no `auth` says the account configures none,
    /// still uncoded, on the `home` route as on the `server` one; a
    /// credential refused says nothing of the kind.
    #[test]
    fn a_401_without_auth_names_the_missing_auth() {
        let (origin, _heads) = answering("401 Unauthorized", "Unauthorized");
        let anonymous = |route: String| {
            let config: CaldavConfig = toml::from_str(&route).unwrap();
            checked("caldav", check_caldav(config, &mut SecretResolver::new()))
        };

        for row in [
            anonymous(format!("server = \"{origin}/\"")),
            anonymous(format!("home = \"{origin}/calendars/alice/\"")),
        ] {
            assert!(!row.ok, "{row:?}");
            assert_eq!(row.code, None, "{row:?}");
            let error = row.error.unwrap_or_default();
            assert!(
                error.contains("answered 401: it asks for credentials")
                    && error.contains("the account configures no `auth`"),
                "{error}"
            );
        }

        let refused = check(format!("server = \"{origin}/\""));
        assert!(!refused.error.unwrap().contains("no `auth`"));
    }

    /// A home answering its PROPFIND passes the check.
    #[test]
    fn a_home_answering_its_propfind_passes() {
        let (origin, heads) = answering("207 Multi-Status", HOME);

        let home = check(format!("home = \"{origin}/calendars/alice/\""));
        assert!(home.ok, "{home:?}");

        let head = heads.recv().unwrap();
        assert!(head.starts_with("PROPFIND /calendars/alice/ "), "{head}");
    }

    /// A server closing the connection the check opened before answering
    /// gets the PROPFIND again on a fresh one, as every io-webdav request
    /// does, and the check passes.
    #[test]
    fn a_home_propfind_is_sent_again_on_a_fresh_connection() {
        let (origin, heads) = closing_first("207 Multi-Status", HOME);

        let home = check(format!("home = \"{origin}/calendars/alice/\""));
        assert!(home.ok, "{home:?}");

        for _ in 0..2 {
            let head = heads.recv().unwrap();
            assert!(head.starts_with("PROPFIND /calendars/alice/ "), "{head}");
        }
    }
}
