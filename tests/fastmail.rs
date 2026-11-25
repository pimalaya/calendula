//! # Fastmail live suite
//!
//! The binary against Fastmail's CalDAV server, on the CalDAV section of
//! config.sample.toml pointed at Fastmail's root with a made-up Basic
//! password, Fastmail's CalDAV server taking Basic only. Only a browser
//! consent issues a Fastmail token, so the suite is `anonymous` only and
//! touches no account: its user is on a reserved domain, no Fastmail
//! account.
//!
//! Run with `cargo test --test fastmail -- --ignored`.

#![cfg(feature = "caldav")]

mod common;

mod anonymous {
    use std::path::{Path, PathBuf};

    use tempfile::TempDir;

    use crate::common::*;

    /// A user no Fastmail account bears, example.org being reserved
    /// (RFC 2606), as a TOML string.
    const USER: &str = r#""alice@example.org""#;

    /// Fastmail's CalDAV root, as a TOML string.
    const SERVER: &str = r#""https://caldav.fastmail.com/dav/""#;

    /// Writes the sample's CalDAV account on Fastmail's root and [`USER`],
    /// its password read through [`TOKEN_COMMAND`].
    fn write_account(dir: &Path) -> PathBuf {
        let section = sample_section("CalDAV", &[("caldav.server", SERVER)]);
        let auth = format!(
            "caldav.auth.basic.username = {USER}\n\
             caldav.auth.basic.password.command = {TOKEN_COMMAND}\n"
        );
        write_config(dir, "fastmail", &format!("{section}{auth}"))
    }

    #[test]
    #[ignore = "requires network and --ignored"]
    fn caldav_account_check_refused() {
        let dir = TempDir::new().unwrap();
        let config = write_account(dir.path());

        let out = calendula(&config, MADE_UP, &["--json", "account", "check"]);
        json_check_failure(&out, "caldav", REFUSED);
    }

    #[test]
    #[ignore = "requires network and --ignored"]
    fn caldav_calendar_list_refused() {
        let dir = TempDir::new().unwrap();
        let config = write_account(dir.path());

        let args = ["--json", "calendar", "list"];
        json_failure(&calendula(&config, MADE_UP, &args), REFUSED);
    }
}
