//! # Event list
//!
//! The built binary listing a vdir calendar of three events two at a
//! time: the header, one line per event and the footer, byte for byte,
//! and the next page resumed through `--page-token`.

#![cfg(feature = "vdir")]

use std::{
    fs,
    path::Path,
    process::{Command, Output, Stdio},
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
fn output(home: &Path, args: &[&str]) -> Output {
    let config = home.join("config.toml");

    Command::new(env!("CARGO_BIN_EXE_calendula"))
        .args(["-c", config.to_str().unwrap()])
        .args(args)
        .env("TZ", "UTC")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

/// Runs the binary as [`output`] does, requiring it to succeed.
fn run(home: &Path, args: &[&str]) -> String {
    let output = output(home, args);
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

    let args = ["event", "list", "-s", "2", "--page-token", "2"];
    assert_eq!(run(home.path(), &args), expected);
}

/// `nextPage` is a token `--page-token` takes back as it is, the page
/// number as a string, on both families.
#[test]
fn a_listing_resumes_at_the_next_page_it_names() {
    let home = account();
    let json = |args: &[&str]| -> Value { serde_json::from_str(&run(home.path(), args)).unwrap() };

    for (family, rows) in [("event", "events"), ("item", "items")] {
        let first = json(&["--json", family, "list", "-s", "2"]);
        assert_eq!(first[rows].as_array().unwrap().len(), 2, "{first}");
        assert_eq!(first["nextPage"], "2", "{first}");

        let token = first["nextPage"].as_str().unwrap();
        let next = json(&["--json", family, "list", "-s", "2", "--page-token", token]);
        assert_eq!(next[rows].as_array().unwrap().len(), 1, "{next}");
        assert_eq!(next[rows][0]["id"], "c", "{next}");
        assert_eq!(next["nextPage"], Value::Null, "{next}");
    }
}

#[test]
fn a_page_token_that_is_no_page_number_is_refused() {
    let home = account();
    let args = ["--json", "event", "list", "-s", "2", "--page-token", "x2"];
    let output = output(home.path(), &args);
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(output.status.code(), Some(1), "{json}");
    assert!(json["error"].as_str().unwrap().contains("`x2`"), "{json}");
}

/// The token wins over a `--page` given beside it, as in the family.
#[test]
fn a_page_token_wins_over_a_page() {
    let home = account();

    for (family, rows) in [("event", "events"), ("item", "items")] {
        let args = ["--json", family, "list", "-s", "2", "-p", "1"];
        let args = [&args[..], &["--page-token", "2"]].concat();
        let json: Value = serde_json::from_str(&run(home.path(), &args)).unwrap();

        assert_eq!(json[rows].as_array().unwrap().len(), 1, "{json}");
        assert_eq!(json[rows][0]["id"], "c", "{json}");
    }
}
