//! # CalDAV ids
//!
//! The built binary naming, on a CalDAV account, a calendar or an item by an
//! id no CalDAV resource can have, one addressing the home set, its parent or
//! the server root: every command fails before the account's secret is
//! resolved or anything connects, coded as a calendar or an item the server
//! does not hold.

#![cfg(feature = "caldav")]

use std::{
    fs,
    path::Path,
    process::{Command, Output, Stdio},
};

use serde_json::Value;
use tempfile::TempDir;

/// An event an `item add` or `item update` writes.
const ICAL: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\n\
                    BEGIN:VEVENT\r\nUID:e1\r\nDTSTAMP:20260101T000000Z\r\n\
                    DTSTART:20260814T090000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

/// A directory holding the configuration of a CalDAV account whose home set
/// no server answers, its password command creating `spawned`: the file
/// says the account was opened.
fn account() -> TempDir {
    let dir = TempDir::new().unwrap();
    let spawned = dir.path().join("spawned").display().to_string();

    let config = format!(
        "[accounts.dav]\n\
         default = true\n\
         caldav.home = \"http://127.0.0.1:9/dav/\"\n\
         caldav.auth.basic.username = \"alice\"\n\
         caldav.auth.basic.password.command = [\"touch\", {spawned:?}]\n"
    );
    fs::write(dir.path().join("config.toml"), config).unwrap();

    dir
}

/// Runs the binary under `--json` with `args` over the account in `dir`.
fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_calendula"))
        .arg("--json")
        .arg("-c")
        .arg(dir.join("config.toml"))
        .args(args)
        .env("HOME", dir)
        .env("XDG_CONFIG_HOME", dir)
        .env_remove("XDG_CONFIG_DIRS")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

/// Fails unless `args` exit 1 with the failure object coded `code` saying
/// `error`, io-webdav's reason in its sources, the account never opened.
fn refused(dir: &Path, args: &[&str], code: Value, error: &str) {
    let out = run(dir, args);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap_or_else(|_| panic!("{stdout}"));

    assert_eq!(out.status.code(), Some(1), "{args:?}: {json}");
    assert_eq!(json["code"], code, "{args:?}: {json}");
    assert_eq!(json["error"], error, "{args:?}: {json}");
    assert!(!dir.join("spawned").exists(), "{args:?} opened the account");
}

/// Whether `json` carries io-webdav's reason for refusing an id.
fn names_webdav_reason(json: &Value) -> bool {
    let sources = json["sources"].as_array().cloned().unwrap_or_default();
    sources
        .iter()
        .filter_map(Value::as_str)
        .any(|source| source.starts_with("WebDAV id "))
}

#[test]
fn a_calendar_id_out_of_the_home_is_not_found() {
    let dir = account();
    let code = Value::from("calendar-not-found");

    for id in ["..", ".", "", "/", "a/..", "%2e%2e", "a//b"] {
        let error = format!("Calendar `{id}` not found");

        let commands: &[&[&str]] = &[
            &["calendar", "update", id, "-n", "Renamed"],
            &["calendar", "delete", id],
            &["event", "list", "-k", id],
            &["event", "agenda", "-k", id],
            &["event", "search", "-k", id, "e1"],
            &["event", "read", "-k", id, "e1.ics"],
            &["event", "add", "-k", id, "--summary", "Bob"],
            &["event", "update", "-k", id, "e1.ics", "--summary", "Bob"],
            &["event", "delete", "-k", id, "e1.ics"],
            &["item", "list", "-k", id],
            &["item", "read", "-k", id, "e1.ics"],
            &["item", "add", "-k", id, ICAL],
            &["item", "update", "-k", id, "e1.ics", ICAL],
            &["item", "delete", "-k", id, "e1.ics"],
            &["caldav", "delete", id],
        ];

        for args in commands {
            refused(dir.path(), args, code.clone(), &error);
        }
    }

    let out = run(dir.path(), &["calendar", "delete", ".."]);
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(names_webdav_reason(&json), "{json}");
}

#[test]
fn an_item_id_out_of_its_calendar_is_not_found() {
    let dir = account();
    let code = Value::from("item-not-found");

    for id in ["..", ".", "", "/", "a/../..", "%2E%2E", "a/"] {
        let error = format!("Item `{id}` not found in calendar `cal`");

        let commands: &[&[&str]] = &[
            &["item", "read", "-k", "cal", id],
            &["item", "update", "-k", "cal", id, ICAL],
            &["item", "delete", "-k", "cal", id],
        ];

        for args in commands {
            refused(dir.path(), args, code.clone(), &error);
        }

        // NOTE: an `event` command names the event in its own sentence.
        let error = format!(
            "Event `{id}` not found in calendar `cal`; an event is named by the id \
             `event list` shows, `event search <UID>` finding it by its iCalendar UID"
        );

        let commands: &[&[&str]] = &[
            &["event", "read", "-k", "cal", id],
            &["event", "update", "-k", "cal", id, "--summary", "Bob"],
            &["event", "delete", "-k", "cal", id],
        ];

        for args in commands {
            refused(dir.path(), args, code.clone(), &error);
        }
    }

    let out = run(dir.path(), &["item", "read", "-k", "cal", ".."]);
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(names_webdav_reason(&json), "{json}");
}

#[test]
fn a_new_calendar_id_out_of_the_home_is_refused() {
    let dir = account();

    for id in ["..", "", "/", "a/.."] {
        let error = format!("Calendar id `{id}` names no CalDAV collection");
        let args = ["calendar", "create", id, "Work"];
        refused(dir.path(), &args, Value::Null, &error);
        refused(dir.path(), &["caldav", "create", id], Value::Null, &error);
    }
}

#[test]
fn a_clean_id_goes_on_to_open_the_account() {
    let dir = account();

    let out = run(dir.path(), &["item", "read", "-k", "My Calendar", "e1.ics"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(dir.path().join("spawned").exists());
}
