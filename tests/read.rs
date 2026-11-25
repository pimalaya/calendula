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
}

#[test]
fn a_read_keeps_the_stored_bytes_in_json() {
    let home = account();

    let stdout = run(home.path(), &["--json", "event", "read", "a"]);
    let json: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["contents"], BODY);
}

/// An id the calendar does not hold fails a read, an update or a delete
/// with a failure object coded `event-not-found`, its sentence Himalaya's
/// for a missing message, saying how an event is named; an update given a
/// source creates nothing, and the backend's own answer is the one source
/// under the sentence.
#[test]
fn an_unknown_id_is_coded_not_found() {
    let home = account();
    let config = home.path().join("config.toml");

    let event = "Event `nope` not found in calendar `personal`; an event is named by \
                 the id `event list` shows, `event read --uid <UID>` reading it by its \
                 iCalendar UID";

    let ical = BODY;
    let commands: &[&[&str]] = &[
        &["event", "update", "nope", ical],
        &["event", "read", "nope"],
        &["event", "update", "nope", "--summary", "Bob"],
        &["event", "delete", "nope"],
    ];

    for args in commands {
        let output = Command::new(env!("CARGO_BIN_EXE_calendula"))
            .args(["-c", config.to_str().unwrap(), "--json"])
            .args(*args)
            .stdin(Stdio::null())
            .output()
            .unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(output.status.code(), Some(1), "{args:?}: {stdout}");
        let json: Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(json["code"], "event-not-found", "{args:?}: {json}");
        assert_eq!(json["error"], event, "{args:?}: {json}");

        let sources = json["sources"].as_array().unwrap();
        assert_eq!(sources.len(), 1, "{args:?}: {json}");
    }

    let written = home.path().join("personal").join("nope.ics");
    assert!(!written.exists(), "an update created the item");
}

/// A read prints the event's resource: its id, its calendar and its
/// version beside its contents, as text, the version the one `event list`
/// shows.
#[test]
fn a_read_prints_the_resource_its_list_names() {
    let home = account();

    let list = run(home.path(), &["--json", "event", "list"]);
    let list: Value = serde_json::from_str(&list).unwrap();
    let row = &list["events"][0];

    let stdout = run(home.path(), &["--json", "event", "read", "a"]);
    let json: Value = serde_json::from_str(&stdout).unwrap();

    for key in ["id", "calendarId", "etag"] {
        assert_eq!(json[key], row[key], "{key}: {json}");
    }

    assert_eq!(json["id"], "a");
    assert_eq!(json["calendarId"], "personal");
    assert_eq!(json["contents"], BODY);
    assert_eq!(json["encoding"], "utf-8");
}

/// A body that is not valid UTF-8 prints as base64 under `--json`, never
/// decoded lossily, and its text view replaces what does not decode.
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

    let stdout = run(home.path(), &["--json", "event", "read", "a"]);
    let json: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["contents"], base64);
    assert_eq!(json["encoding"], "base64");

    let text = run(home.path(), &["event", "read", "a"]);
    assert!(text.contains("SUMMARY:Caf\u{fffd}\n"), "{text}");
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
