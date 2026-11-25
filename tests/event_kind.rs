//! # Event kind
//!
//! The built binary over a vdir calendar holding an event and a to-do:
//! the `event` commands list, read, update and delete the event alone,
//! the to-do refused `event-not-found` and left byte for byte, and a
//! to-do source refused `event-invalid`.

#![cfg(feature = "vdir")]

use std::{
    fs,
    path::Path,
    process::{Command, Output, Stdio},
};

use serde_json::Value;
use tempfile::TempDir;

/// The event `a` the calendar holds.
const EVENT: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\n\
    BEGIN:VEVENT\r\nUID:a@example.org\r\nDTSTAMP:20261101T000000Z\r\n\
    DTSTART:20261112T090000Z\r\nDTEND:20261112T093000Z\r\nSUMMARY:Standup\r\n\
    END:VEVENT\r\nEND:VCALENDAR\r\n";

/// The to-do `t` the calendar holds beside the event.
const TODO: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\n\
    BEGIN:VTODO\r\nUID:t@example.org\r\nDTSTAMP:20261101T000000Z\r\n\
    DUE:20261113T170000Z\r\nSUMMARY:Pay the rent\r\nEND:VTODO\r\n\
    END:VCALENDAR\r\n";

/// An event an update would write over the to-do.
const SOURCE: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\n\
    BEGIN:VEVENT\r\nUID:t@example.org\r\nDTSTAMP:20261101T000000Z\r\n\
    DTSTART:20261113T170000Z\r\nSUMMARY:Rent\r\nEND:VEVENT\r\n\
    END:VCALENDAR\r\n";

/// A vdir account holding the calendar `personal`, the event `a` and the
/// to-do `t` in it.
fn account() -> TempDir {
    let home = TempDir::new().unwrap();
    let calendar = home.path().join("personal");
    fs::create_dir(&calendar).unwrap();
    fs::write(calendar.join("a.ics"), EVENT).unwrap();
    fs::write(calendar.join("t.ics"), TODO).unwrap();

    let config = format!(
        "[accounts.perso]\ndefault = true\ncalendar.default = \"personal\"\n\
         vdir.root = {:?}\n",
        home.path().display().to_string()
    );
    fs::write(home.path().join("config.toml"), config).unwrap();

    home
}

/// Runs the binary with `--json` and `args` over the account in `home`.
fn run(home: &Path, args: &[&str]) -> Output {
    let config = home.join("config.toml");
    Command::new(env!("CARGO_BIN_EXE_calendula"))
        .args(["-c", config.to_str().unwrap(), "--json"])
        .args(args)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

#[test]
fn a_listing_shows_the_event_alone() {
    let home = account();

    let output = run(home.path(), &["event", "list"]);
    assert!(output.status.success(), "{output:?}");
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let ids: Vec<&str> = json["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|event| event["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["a"], "{json}");
}

/// Reading, updating (from flags or a source) and deleting the to-do
/// through `event` fail `event-not-found`, its file left as it was.
#[test]
fn a_to_do_is_no_event() {
    let home = account();
    let todo = home.path().join("personal").join("t.ics");

    let commands: &[&[&str]] = &[
        &["event", "read", "t"],
        &["event", "update", "t", "--summary", "Rent"],
        &["event", "update", "t", SOURCE],
        &["event", "delete", "t"],
    ];

    for args in commands {
        let output = run(home.path(), args);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(output.status.code(), Some(1), "{args:?}: {stdout}");
        let json: Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(json["code"], "event-not-found", "{args:?}: {json}");
        assert_eq!(fs::read_to_string(&todo).unwrap(), TODO, "{args:?}");
    }
}

/// A source holding no VEVENT is refused `event-invalid` by `event add`
/// and `event update`: neither writes a to-do, the calendar left as it
/// was.
#[test]
fn a_to_do_source_is_refused() {
    let home = account();
    let calendar = home.path().join("personal");
    let other = TODO.replace("t@example.org", "u@example.org");

    let commands: &[&[&str]] = &[&["event", "add", &other], &["event", "update", "a", &other]];

    for args in commands {
        let output = run(home.path(), args);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(output.status.code(), Some(1), "{args:?}: {stdout}");
        let json: Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(json["code"], "event-invalid", "{args:?}: {json}");
    }

    assert_eq!(fs::read_dir(&calendar).unwrap().count(), 2);
    assert_eq!(fs::read_to_string(calendar.join("a.ics")).unwrap(), EVENT);
}
