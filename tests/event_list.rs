//! # Event list
//!
//! The built binary listing a vdir calendar of three events two at a
//! time: the header, one line per event and the footer, byte for byte.

#![cfg(feature = "vdir")]

use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

use serde_json::Value;
use tempfile::TempDir;

/// Writes one event `id` to the calendar `dir`.
fn event(dir: &Path, id: &str, start: &str, end: &str, summary: &str) {
    let ics = format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\nBEGIN:VEVENT\r\n\
         UID:{id}@example.org\r\nDTSTAMP:20261101T000000Z\r\nDTSTART:{start}\r\n\
         DTEND:{end}\r\nSUMMARY:{summary}\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
    );
    fs::write(dir.join(format!("{id}.ics")), ics).unwrap();
}

/// A vdir account `perso` holding the calendar `personal`, and its
/// configuration file.
fn account() -> TempDir {
    let home = TempDir::new().unwrap();
    let calendar = home.path().join("personal");
    fs::create_dir(&calendar).unwrap();

    event(
        &calendar,
        "a",
        "20261112T090000Z",
        "20261112T093000Z",
        "Standup",
    );
    event(
        &calendar,
        "b",
        "20261113T140000Z",
        "20261113T150000Z",
        "Review",
    );
    event(&calendar, "c", "20261114T100000Z", "20261114T110000Z", "");

    let config = format!(
        "[accounts.perso]\ndefault = true\ncalendar.default = \"personal\"\n\
         vdir.home-dir = {:?}\n",
        home.path().display().to_string()
    );
    fs::write(home.path().join("config.toml"), config).unwrap();

    home
}

/// Runs the binary with `args` over the account in `home`, in UTC.
fn run(home: &Path, args: &[&str]) -> String {
    let config = home.join("config.toml");
    let output = Command::new(env!("CARGO_BIN_EXE_calendula"))
        .args(["-c", config.to_str().unwrap()])
        .args(args)
        .env("TZ", "UTC")
        .stdin(Stdio::null())
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(output.status.success(), "{stdout}");
    stdout
}

#[test]
fn an_event_list_prints_a_header_one_line_per_event_and_a_footer() {
    let home = account();

    let expected = "\
Account:    perso (vdir)
Calendar:   personal

a  2026-11-12 09:00  2026-11-12 09:30  Standup
b  2026-11-13 14:00  2026-11-13 15:00  Review

Page:       1, events 1 to 2
Next page:  --page 2
";
    let args = ["event", "list", "-s", "2"];
    assert_eq!(run(home.path(), &args), expected);

    let expected = "\
Account:    perso (vdir)
Calendar:   personal

c  2026-11-14 10:00  2026-11-14 11:00  -

Page:       2, event 3, the last
";
    let args = ["event", "list", "-s", "2", "-p", "2"];
    assert_eq!(run(home.path(), &args), expected);
}

#[test]
fn an_event_list_names_the_next_page_in_json() {
    let home = account();
    let args = ["--json", "event", "list", "-s", "2"];
    let json: Value = serde_json::from_str(&run(home.path(), &args)).unwrap();

    assert_eq!(json["events"].as_array().unwrap().len(), 2);
    assert_eq!(json["nextPage"], 2);
}
