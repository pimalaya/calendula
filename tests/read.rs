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
         vdir.root = {:?}\n",
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

/// An id the calendar does not hold fails the read with a failure object
/// coded `item-not-found`.
#[test]
fn an_unknown_id_is_coded_not_found() {
    let home = account();
    let config = home.path().join("config.toml");

    for family in ["event", "item"] {
        let output = Command::new(env!("CARGO_BIN_EXE_calendula"))
            .args([
                "-c",
                config.to_str().unwrap(),
                "--json",
                family,
                "read",
                "nope",
            ])
            .stdin(Stdio::null())
            .output()
            .unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(output.status.code(), Some(1), "{family}: {stdout}");
        let json: Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(json["code"], "item-not-found", "{family}: {json}");
    }
}

/// Both reads print the row `item list` prints: the item's id, its
/// calendar and its version beside its contents, as text.
#[test]
fn a_read_prints_the_row_its_list_prints() {
    let home = account();

    let list = run(home.path(), &["--json", "item", "list"]);
    let list: Value = serde_json::from_str(&list).unwrap();
    let row = &list["items"][0];

    for family in ["event", "item"] {
        let stdout = run(home.path(), &["--json", family, "read", "a"]);
        let json: Value = serde_json::from_str(&stdout).unwrap();

        for key in ["id", "calendarId", "etag", "contents", "encoding"] {
            assert_eq!(json[key], row[key], "{family} {key}: {json}");
        }

        assert_eq!(json["id"], "a", "{family}");
        assert_eq!(json["calendarId"], "personal", "{family}");
        assert_eq!(json["encoding"], "utf-8", "{family}");
    }
}

/// A body that is not valid UTF-8 prints as base64 under `--json`, never
/// decoded lossily, in a listing and in both reads, and its text view
/// replaces what does not decode.
#[test]
fn a_body_that_is_not_utf8_is_base64_in_json() {
    use base64_codec::codec::Base64Codec;

    let body: &[u8] = b"BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\n\
        BEGIN:VEVENT\r\nUID:l@example.org\r\nDTSTAMP:20261101T000000Z\r\n\
        DTSTART:20261112T090000Z\r\nSUMMARY:Caf\xe9\r\nEND:VEVENT\r\n\
        END:VCALENDAR\r\n";

    let home = account();
    fs::write(home.path().join("personal").join("a.ics"), body).unwrap();

    let mut base64 = Vec::new();
    Base64Codec::default().encode_vec(body, &mut base64);
    let base64 = String::from_utf8(base64).unwrap();

    let list = run(home.path(), &["--json", "item", "list"]);
    let list: Value = serde_json::from_str(&list).unwrap();
    assert_eq!(list["items"][0]["contents"], base64, "{list}");
    assert_eq!(list["items"][0]["encoding"], "base64", "{list}");

    for family in ["event", "item"] {
        let stdout = run(home.path(), &["--json", family, "read", "a"]);
        let json: Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(json["contents"], base64, "{family}");
        assert_eq!(json["encoding"], "base64", "{family}");

        let text = run(home.path(), &[family, "read", "a"]);
        assert!(text.contains("SUMMARY:Caf\u{fffd}\n"), "{family}: {text}");
    }
}

/// An event carrying no `DESCRIPTION` nor `LOCATION` prints them `null`,
/// an empty string meaning the empty value.
#[test]
fn an_absent_description_or_location_is_null() {
    let home = account();
    let body = BODY
        .replace("LOCATION:Room\u{202e}1\r\n", "")
        .replace("DESCRIPTION:Agenda\r\n\tand notes\r\n", "");
    fs::write(home.path().join("personal").join("a.ics"), body).unwrap();

    let stdout = run(home.path(), &["--json", "event", "read", "a"]);
    let json: Value = serde_json::from_str(&stdout).unwrap();
    let event = &json["events"][0];

    assert_eq!(event["summary"], "Standup\x1b]52;c;AAAA\x07", "{json}");
    assert!(event["description"].is_null(), "{json}");
    assert!(event["location"].is_null(), "{json}");
}
