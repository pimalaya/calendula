//! # Read
//!
//! The built binary reading a vdir event whose body carries an escape
//! sequence, a bidi override and CRLF line ends: defused in text, its bytes
//! kept under `--json`.

#![cfg(feature = "vdir")]

use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

use serde_json::Value;
use tempfile::TempDir;

/// The stored body: an OSC 52 clipboard write in the summary, a bidi
/// override in the location, a description folded on a tab.
const BODY: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\n\
    BEGIN:VEVENT\r\nUID:a@example.org\r\nDTSTAMP:20261101T000000Z\r\n\
    DTSTART:20261112T090000Z\r\nSUMMARY:Standup\x1b]52;c;AAAA\x07\r\n\
    LOCATION:Room\u{202e}1\r\nDESCRIPTION:Agenda\r\n\tand notes\r\n\
    END:VEVENT\r\nEND:VCALENDAR\r\n";

/// The text both reads print.
const PRINTED: &str = "\
BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//test//EN
BEGIN:VEVENT
UID:a@example.org
DTSTAMP:20261101T000000Z
DTSTART:20261112T090000Z
SUMMARY:Standup\u{fffd}]52;c;AAAA\u{fffd}
LOCATION:Room\u{fffd}1
DESCRIPTION:Agenda
\tand notes
END:VEVENT
END:VCALENDAR
";

/// A vdir account holding the calendar `personal`, its one event `a`
/// stored as [`BODY`], and its configuration file.
fn account() -> TempDir {
    let home = TempDir::new().unwrap();
    let calendar = home.path().join("personal");
    fs::create_dir(&calendar).unwrap();
    fs::write(calendar.join("a.ics"), BODY).unwrap();

    let config = format!(
        "[accounts.perso]\ndefault = true\ncalendar.default = \"personal\"\n\
         vdir.home-dir = {:?}\n",
        home.path().display().to_string()
    );
    fs::write(home.path().join("config.toml"), config).unwrap();

    home
}

/// Runs the binary with `args` over the account in `home`.
fn run(home: &Path, args: &[&str]) -> String {
    let config = home.join("config.toml");
    let output = Command::new(env!("CARGO_BIN_EXE_calendula"))
        .args(["-c", config.to_str().unwrap()])
        .args(args)
        .stdin(Stdio::null())
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(output.status.success(), "{stdout}");
    stdout
}

#[test]
fn a_read_prints_the_stored_body_defused() {
    let home = account();

    assert_eq!(run(home.path(), &["event", "read", "a"]), PRINTED);
    assert_eq!(run(home.path(), &["item", "read", "a"]), PRINTED);
}

#[test]
fn a_read_keeps_the_stored_bytes_in_json() {
    let home = account();

    for family in ["event", "item"] {
        let stdout = run(home.path(), &["--json", family, "read", "a"]);
        let json: Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(json["contents"], BODY, "{family}");
    }
}
