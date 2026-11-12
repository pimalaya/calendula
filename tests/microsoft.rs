//! # Microsoft live suite
//!
//! The binary against Microsoft Graph. `anonymous` runs the Microsoft
//! Graph section of config.sample.toml with a made-up token, touching no
//! account. `credentialed` reads the test tenant's calendars with an
//! app-only token the suite mints through the client credentials grant
//! of a confidential registration, read from
//! `CALENDULA_MICROSOFT_TENANT_ID`, `CALENDULA_MICROSOFT_CLIENT_ID` and
//! `CALENDULA_MICROSOFT_CLIENT_SECRET`; `CALENDULA_MICROSOFT_USER` names
//! the user. It only reads.
//!
//! Run with `cargo test --test microsoft -- --ignored`. A failure never
//! shows a token: the binary reads it from its environment only.

#![cfg(feature = "msgraph")]

mod common;

mod anonymous {
    use std::path::{Path, PathBuf};

    use tempfile::TempDir;

    use crate::common::*;

    /// Writes the sample's Graph account, its token read through
    /// [`TOKEN_COMMAND`].
    fn write_account(dir: &Path) -> PathBuf {
        let token = [("msgraph.auth.token.command", TOKEN_COMMAND)];
        let section = sample_section("Microsoft Graph", &token);
        write_config(dir, "microsoft", &section)
    }

    #[test]
    #[ignore = "requires network and --ignored"]
    fn graph_account_check_refused() {
        let dir = TempDir::new().unwrap();
        let config = write_account(dir.path());

        let out = calendula(&config, MADE_UP, &["--json", "account", "check"]);
        json_check_failure(&out, "msgraph", REFUSED);
    }

    #[test]
    #[ignore = "requires network and --ignored"]
    fn graph_calendar_list_refused() {
        let dir = TempDir::new().unwrap();
        let config = write_account(dir.path());

        let args = ["--json", "calendar", "list"];
        json_failure(&calendula(&config, MADE_UP, &args), REFUSED);
    }
}

mod credentialed {
    use std::{
        path::{Path, PathBuf},
        sync::OnceLock,
    };

    use io_oauth::{
        client::Oauth20ClientStd,
        rfc6749::client_credentials::Oauth20ClientCredentialsRequestParams,
    };
    use pimalaya_stream::tls::Tls;
    use secrecy::{ExposeSecret, SecretString};
    use serde_json::Value;
    use tempfile::TempDir;

    use crate::common::*;

    /// The user read when `CALENDULA_MICROSOFT_USER` is unset: an app-only
    /// token has no `/me`.
    const USER: &str = "microsoft@pimalaya.onmicrosoft.com";

    /// The scope of an app-only Graph token.
    const SCOPE: &str = "https://graph.microsoft.com/.default";

    /// The name Exchange gives the default calendar of an English
    /// mailbox, `calendar list` reading `default: false` on Graph.
    const DEFAULT_CALENDAR: &str = "Calendar";

    /// A Graph token of the test registration, through the client
    /// credentials grant, minted once per run.
    fn token() -> &'static str {
        static TOKEN: OnceLock<String> = OnceLock::new();

        TOKEN.get_or_init(|| {
            let [tenant, client_id, secret] = require_env([
                "CALENDULA_MICROSOFT_TENANT_ID",
                "CALENDULA_MICROSOFT_CLIENT_ID",
                "CALENDULA_MICROSOFT_CLIENT_SECRET",
            ]);
            let endpoint = format!("https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token");

            let tls = Tls::default();
            let mut client = Oauth20ClientStd::connect(endpoint.parse().unwrap(), &tls, client_id)
                .unwrap_or_else(|err| panic!("cannot reach {endpoint}: {err}"));
            client.client_secret = Some(SecretString::from(secret));

            let params = Oauth20ClientCredentialsRequestParams {
                scope: [SCOPE.into()].into(),
            };

            match client.request_client_credentials(params) {
                Ok(Ok(success)) => success.access_token.expose_secret().to_owned(),
                Ok(Err(err)) => panic!(
                    "token refused: {:?}: {:?}",
                    err.error, err.error_description
                ),
                Err(err) => panic!("token request failed: {err}"),
            }
        })
    }

    /// The user the suite reads, as a TOML string.
    fn user() -> String {
        let user = env_var("CALENDULA_MICROSOFT_USER").unwrap_or_else(|| USER.to_owned());
        format!("\"{user}\"")
    }

    /// Writes the sample's Graph account on the test user, its token read
    /// through [`TOKEN_COMMAND`].
    fn write_account(dir: &Path) -> PathBuf {
        let user = user();
        let overrides = [
            ("msgraph.auth.token.command", TOKEN_COMMAND),
            ("msgraph.user-id", user.as_str()),
        ];

        let section = sample_section("Microsoft Graph", &overrides);
        write_config(dir, "microsoft", &section)
    }

    /// The `calendar list` of the account `config` names, checked.
    fn calendars(config: &Path) -> Value {
        let out = calendula(config, token(), &["--json", "calendar", "list"]);
        let list = json(&out, 0);
        assert_calendars(&list);
        list
    }

    #[test]
    #[ignore = "requires Microsoft credentials and --ignored"]
    fn graph_account_check() {
        let dir = TempDir::new().unwrap();
        let config = write_account(dir.path());

        let out = calendula(&config, token(), &["--json", "account", "check"]);
        assert_check(&json(&out, 0), "msgraph");
    }

    #[test]
    #[ignore = "requires Microsoft credentials and --ignored"]
    fn graph_calendar_list() {
        let dir = TempDir::new().unwrap();
        let config = write_account(dir.path());

        let list = calendars(&config);
        let calendars = list["calendars"].as_array().unwrap();
        let names: Vec<_> = calendars.iter().map(|c| &c["name"]).collect();
        let found = names.iter().any(|name| *name == DEFAULT_CALENDAR);
        assert!(found, "no calendar {DEFAULT_CALENDAR} among {names:?}");
    }

    #[test]
    #[ignore = "requires Microsoft credentials and --ignored"]
    fn graph_event_list() {
        let dir = TempDir::new().unwrap();
        let config = write_account(dir.path());

        let list = calendars(&config);
        let calendars = list["calendars"].as_array().unwrap();
        let default = calendars.iter().find(|c| c["name"] == DEFAULT_CALENDAR);
        let default = default.unwrap_or_else(|| panic!("no calendar {DEFAULT_CALENDAR}"));
        let id = default["id"].as_str().unwrap();

        let args = ["--json", "event", "list", "-s", "5", "-k", id];
        let out = calendula(&config, token(), &args);
        assert_events(&json(&out, 0), 5);
    }
}
