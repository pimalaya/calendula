//! # Google live suite
//!
//! The binary against the Google Calendar API and Google's CalDAV server.
//! `anonymous` runs a made-up token, touching no account. `credentialed`
//! reads the test Workspace's calendars with a token the suite mints for a
//! service account acting as `CALENDULA_GOOGLE_SERVICE_ACCOUNT_SUBJECT`
//! (RFC 7523 section 2.1), its key read from
//! `CALENDULA_GOOGLE_SERVICE_ACCOUNT_KEY` or the file
//! `CALENDULA_GOOGLE_SERVICE_ACCOUNT_KEY_FILE` names. It only reads, and
//! only the Calendar API: Google's CalDAV server refuses a token of the
//! narrow scopes the delegation grants (HTTP 403, insufficient
//! permission), so CalDAV is covered anonymously only.
//!
//! Both accounts are sections of config.sample.toml: Google Calendar, and
//! Google CalDAV on its principal.
//!
//! Run with `cargo test --test google -- --ignored`. A failure never shows
//! a token: the binary reads it from its environment only.

#![cfg(all(feature = "gcal", feature = "caldav"))]

mod common;

use std::path::{Path, PathBuf};

use crate::common::*;

/// Writes the sample's Google Calendar account, its token read through
/// [`TOKEN_COMMAND`].
fn write_gcal_account(dir: &Path) -> PathBuf {
    let token = [("gcal.auth.token.command", TOKEN_COMMAND)];
    write_config(dir, "google", &sample_section("Google Calendar", &token))
}

/// Writes the sample's Google CalDAV account, its bearer token read
/// through [`TOKEN_COMMAND`].
fn write_caldav_account(dir: &Path) -> PathBuf {
    let token = [("caldav.auth.bearer.token.command", TOKEN_COMMAND)];
    write_config(dir, "google", &sample_section("Google CalDAV", &token))
}

mod anonymous {
    use tempfile::TempDir;

    use crate::{common::*, write_caldav_account, write_gcal_account};

    #[test]
    #[ignore = "requires network and --ignored"]
    fn gcal_account_check_refused() {
        let dir = TempDir::new().unwrap();
        let config = write_gcal_account(dir.path());

        let out = calendula(&config, MADE_UP, &["--json", "account", "check"]);
        json_check_failure(&out, "gcal", REFUSED);
    }

    #[test]
    #[ignore = "requires network and --ignored"]
    fn gcal_calendar_list_refused() {
        let dir = TempDir::new().unwrap();
        let config = write_gcal_account(dir.path());

        let args = ["--json", "calendar", "list"];
        json_failure(&calendula(&config, MADE_UP, &args), REFUSED);
    }

    #[test]
    #[ignore = "requires network and --ignored"]
    fn caldav_account_check_refused() {
        let dir = TempDir::new().unwrap();
        let config = write_caldav_account(dir.path());

        let out = calendula(&config, MADE_UP, &["--json", "account", "check"]);
        json_check_failure(&out, "caldav", REFUSED);
    }

    #[test]
    #[ignore = "requires network and --ignored"]
    fn caldav_calendar_list_refused() {
        let dir = TempDir::new().unwrap();
        let config = write_caldav_account(dir.path());

        let args = ["--json", "calendar", "list"];
        json_failure(&calendula(&config, MADE_UP, &args), REFUSED);
    }
}

mod credentialed {
    use std::{collections::BTreeSet, sync::OnceLock, time::Duration};

    use io_oauth::{
        client::Oauth20ClientStd,
        rfc7523::{
            assertion::{Oauth20JwtBearerClaims, Oauth20JwtBearerKey},
            auth_grant::Oauth20JwtBearerGrantRequestParams,
        },
    };
    use pimalaya_stream::tls::Tls;
    use secrecy::ExposeSecret;
    use serde_json::Value;
    use tempfile::TempDir;

    use crate::{common::*, write_gcal_account};

    /// The user acted as when `CALENDULA_GOOGLE_SERVICE_ACCOUNT_SUBJECT`
    /// is unset.
    const SUBJECT: &str = "google@pimalaya.org";

    /// The scopes a read of the Google Calendar API needs, the narrowest
    /// the delegation grants: events, and the calendar list read-only.
    /// One scope it does not grant fails the whole request, full
    /// `calendar` among them.
    const SCOPES: [&str; 2] = [
        "https://www.googleapis.com/auth/calendar.events",
        "https://www.googleapis.com/auth/calendar.calendarlist.readonly",
    ];

    /// The variable holding the service account key.
    const KEY_VAR: &str = "CALENDULA_GOOGLE_SERVICE_ACCOUNT_KEY";

    /// The user the service account acts as, whose primary calendar
    /// bears its address as id.
    fn subject() -> String {
        env_var("CALENDULA_GOOGLE_SERVICE_ACCOUNT_SUBJECT").unwrap_or_else(|| SUBJECT.to_owned())
    }

    /// A token of the service account acting as [`subject`], minted once
    /// per run. A failure never shows the key, nor what it signed.
    fn token() -> &'static str {
        static TOKEN: OnceLock<String> = OnceLock::new();

        TOKEN.get_or_init(|| {
            let Some(key) = env_content(KEY_VAR) else {
                panic!("set {KEY_VAR} or {KEY_VAR}_FILE to run this test");
            };
            let key: Value = serde_json::from_str(&key)
                .unwrap_or_else(|_| panic!("{KEY_VAR} is not a service account key"));
            let (Some(email), Some(pem), Some(endpoint)) = (
                key["client_email"].as_str(),
                key["private_key"].as_str(),
                key["token_uri"].as_str(),
            ) else {
                panic!("{KEY_VAR} is not a service account key");
            };
            let signer = Oauth20JwtBearerKey::from_pkcs8_pem(pem)
                .unwrap_or_else(|_| panic!("{KEY_VAR} holds no PKCS#8 key"));

            let tls = Tls::default();
            let client = Oauth20ClientStd::connect(endpoint.parse().unwrap(), &tls, email);
            let mut client = client.unwrap_or_else(|err| panic!("cannot reach {endpoint}: {err}"));

            let subject = subject();
            let claims = Oauth20JwtBearerClaims {
                iss: email.into(),
                sub: Some(subject.as_str().into()),
                scope: SCOPES.map(Into::into).into(),
                ..Default::default()
            };
            let validity = Duration::from_secs(600);
            let assertion = client
                .sign_jwt_bearer_assertion(&signer, claims, None, validity)
                .unwrap_or_else(|_| panic!("cannot sign the assertion"));

            let params = Oauth20JwtBearerGrantRequestParams {
                assertion,
                scope: BTreeSet::new(),
            };

            match client.request_jwt_bearer_grant(params) {
                Ok(Ok(success)) => success.access_token.expose_secret().to_owned(),
                Ok(Err(err)) => panic!(
                    "token refused: {:?}: {:?}",
                    err.error, err.error_description
                ),
                Err(err) => panic!("token request failed: {err}"),
            }
        })
    }

    #[test]
    #[ignore = "requires Google credentials and --ignored"]
    fn gcal_account_check() {
        let dir = TempDir::new().unwrap();
        let config = write_gcal_account(dir.path());

        let out = calendula(&config, token(), &["--json", "account", "check"]);
        assert_check(&json(&out, 0), "gcal");
    }

    #[test]
    #[ignore = "requires Google credentials and --ignored"]
    fn gcal_calendar_list() {
        let dir = TempDir::new().unwrap();
        let config = write_gcal_account(dir.path());

        let out = calendula(&config, token(), &["--json", "calendar", "list"]);
        let ids = assert_calendars(&json(&out, 0));

        let primary = subject();
        assert!(ids.contains(&primary), "no primary calendar among {ids:?}");
    }

    #[test]
    #[ignore = "requires Google credentials and --ignored"]
    fn gcal_event_list() {
        let dir = TempDir::new().unwrap();
        let config = write_gcal_account(dir.path());

        let primary = subject();
        let args = ["--json", "event", "list", "-s", "5", "-k", &primary];
        let out = calendula(&config, token(), &args);
        assert_events(&json(&out, 0), 5);
    }
}
