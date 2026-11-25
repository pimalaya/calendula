//! # Event list
//!
//! The built binary listing a vdir calendar of three events two at a
//! time: the header, then one line per event, byte for byte, and the next
//! page resumed through `--page-token`; and the same calendar holding an
//! item that does not parse and one that cannot be read, named last and
//! exiting 3.

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
         vdir.root = {:?}\n",
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
fn an_event_list_prints_a_header_then_one_line_per_event() {
    let home = account();

    let expected = "\
Account:   perso (vdir)
Calendar:  personal
Page:      1, events 1 to 2

a  2026-11-12 09:00  2026-11-12 09:30  Standup
b  2026-11-13 14:00  2026-11-13 15:00  Review

Next page: --page 2
";
    let args = ["event", "list", "-s", "2"];
    assert_eq!(run(home.path(), &args), expected);

    let expected = "\
Account:   perso (vdir)
Calendar:  personal
Page:      2, event 3, the last

c  2026-11-14 10:00  2026-11-14 11:00  -
";
    let args = ["event", "list", "-s", "2", "-p", "2"];
    assert_eq!(run(home.path(), &args), expected);

    let args = ["event", "list", "-s", "2", "--page-token", "2"];
    assert_eq!(run(home.path(), &args), expected);
}

/// `nextPage` is a token `--page-token` takes back as it is, the page
/// number as a string.
#[test]
fn a_listing_resumes_at_the_next_page_it_names() {
    let home = account();
    let json = |args: &[&str]| -> Value { serde_json::from_str(&run(home.path(), args)).unwrap() };

    let first = json(&["--json", "event", "list", "-s", "2"]);
    assert_eq!(first["events"].as_array().unwrap().len(), 2, "{first}");
    assert_eq!(first["nextPage"], "2", "{first}");

    let token = first["nextPage"].as_str().unwrap();
    let next = json(&["--json", "event", "list", "-s", "2", "--page-token", token]);
    assert_eq!(next["events"].as_array().unwrap().len(), 1, "{next}");
    assert_eq!(next["events"][0]["id"], "c", "{next}");
    assert_eq!(next["nextPage"], Value::Null, "{next}");
}

#[test]
fn a_page_token_that_is_no_page_number_is_refused() {
    let home = account();
    let args = ["--json", "event", "list", "-s", "2", "--page-token", "x2"];
    let output = output(home.path(), &args);
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(output.status.code(), Some(1), "{json}");
    assert_eq!(json["code"], "page-token-invalid", "{json}");
    assert!(json["error"].as_str().unwrap().contains("x2"), "{json}");
}

/// `event read --uid` reads the item carrying the UID, as a read by its
/// id does; an unknown UID is `event-not-found`, and `event search` takes
/// a query, which only a pimdir store's index answers.
#[test]
fn a_uid_is_read_by_event_read_and_a_search_takes_a_query() {
    let home = account();
    let json = |args: &[&str]| -> Value { serde_json::from_str(&run(home.path(), args)).unwrap() };

    let by_uid = json(&["--json", "event", "read", "--uid", "a@example.org"]);
    let by_id = json(&["--json", "event", "read", "a"]);
    assert_eq!(by_uid, by_id);
    assert_eq!(by_uid["events"][0]["calendarId"], "personal", "{by_uid}");

    let args = ["--json", "event", "read", "--uid", "nope@example.org"];
    let missing = output(home.path(), &args);
    let error: Value = serde_json::from_slice(&missing.stdout).unwrap();
    assert_eq!(missing.status.code(), Some(1), "{error}");
    assert_eq!(error["code"], "event-not-found", "{error}");

    let args = ["--json", "event", "read", "a", "--uid", "a@example.org"];
    assert_eq!(output(home.path(), &args).status.code(), Some(2));

    if !cfg!(feature = "pimdir") {
        return;
    }
    let refused = output(home.path(), &["--json", "event", "search", "standup"]);
    let error: Value = serde_json::from_slice(&refused.stdout).unwrap();
    assert_eq!(refused.status.code(), Some(1), "{error}");
    assert_eq!(error["code"], "capability-missing", "{error}");
}

/// The event commands take the id a listing shows, never a UID: `event
/// delete` given one refuses it `event-not-found`, naming `event read --uid`,
/// and deletes nothing.
#[test]
fn a_uid_is_no_event_id() {
    let home = account();
    let output = output(home.path(), &["--json", "event", "delete", "a@example.org"]);
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(output.status.code(), Some(1), "{json}");
    assert_eq!(json["code"], "event-not-found", "{json}");
    let error = json["error"].as_str().unwrap();
    assert!(error.contains("`event read --uid <UID>`"), "{error}");
    assert!(home.path().join("personal").join("a.ics").is_file());
}

/// The token wins over a `--page` given beside it, as in the family.
#[test]
fn a_page_token_wins_over_a_page() {
    let home = account();

    let args = [
        "--json",
        "event",
        "list",
        "-s",
        "2",
        "-p",
        "1",
        "--page-token",
        "2",
    ];
    let json: Value = serde_json::from_str(&run(home.path(), &args)).unwrap();

    assert_eq!(json["events"].as_array().unwrap().len(), 1, "{json}");
    assert_eq!(json["events"][0]["id"], "c", "{json}");
}

/// The account of [`account`], its calendar also holding `d`, which is no
/// iCalendar, and on unix `e`, an event no one may read. Returns the path
/// of `e`, `None` where it reads anyway (not unix, or run as root).
fn unreadable_account() -> (TempDir, Option<String>) {
    let home = account();
    let calendar = home.path().join("personal");
    fs::write(calendar.join("d.ics"), "not a calendar").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let path = calendar.join("e.ics");
        event(
            &calendar,
            "e",
            "20261115T100000Z",
            "20261115T110000Z",
            "Gone",
        );
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();

        if fs::read(&path).is_err() {
            return (home, Some(path.display().to_string()));
        }

        fs::remove_file(&path).unwrap();
    }

    (home, None)
}

/// An item that does not parse prints no event but an `event-unreadable`
/// entry, and so does a file that cannot be read; the other events list,
/// the entries print last and the command exits 3.
#[test]
fn an_unreadable_item_is_named_last_and_exits_3() {
    let (home, unread) = unreadable_account();

    let output = output(home.path(), &["--json", "event", "list", "-s", "0"]);
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(3), "{json}");

    let ids: Vec<&str> = json["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|event| event["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["a", "b", "c"], "{json}");

    let errors = json["errors"].as_array().unwrap();
    let parsed = errors.last().unwrap();
    assert_eq!(parsed["id"], "d", "{json}");
    assert_eq!(parsed["code"], "event-unreadable", "{json}");
    assert_eq!(
        parsed["error"], "Cannot parse item `d` as iCalendar",
        "{json}"
    );
    assert_eq!(parsed["sources"].as_array().unwrap().len(), 1, "{json}");

    let mut lines = String::new();

    if let Some(path) = &unread {
        assert_eq!(errors.len(), 2, "{json}");
        assert_eq!(errors[0]["id"], "e", "{json}");
        assert_eq!(errors[0]["code"], "event-unreadable", "{json}");
        let error = format!("Cannot read item file {path}");
        assert_eq!(errors[0]["error"], error, "{json}");
        assert_eq!(errors[0]["sources"].as_array().unwrap().len(), 1, "{json}");
        lines.push_str(&format!("Error:  e  event-unreadable  {error}\n"));
    } else {
        assert_eq!(errors.len(), 1, "{json}");
    }

    lines.push_str("Error:  d  event-unreadable  Cannot parse item `d` as iCalendar\n");

    let output = self::output(home.path(), &["event", "list", "-s", "0"]);
    assert_eq!(output.status.code(), Some(3));

    let expected = format!(
        "\
Account:   perso (vdir)
Calendar:  personal
Page:      all, events 1 to 3

a  2026-11-12 09:00  2026-11-12 09:30  Standup
b  2026-11-13 14:00  2026-11-13 15:00  Review
c  2026-11-14 10:00  2026-11-14 11:00  -

{lines}"
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
}

/// A file that cannot be read holds no page, so it prints on the first
/// alone; an item that does not parse prints on the page holding it.
#[test]
fn an_entry_prints_on_its_page() {
    let (home, unread) = unreadable_account();
    let json = |page: &str| -> (Option<i32>, Value) {
        let args = ["--json", "event", "list", "-s", "2", "-p", page];
        let output = output(home.path(), &args);
        let json = serde_json::from_slice(&output.stdout).unwrap();
        (output.status.code(), json)
    };

    let (code, first) = json("1");
    let ids: Vec<&str> = first["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|error| error["id"].as_str().unwrap())
        .collect();

    match unread {
        Some(_) => {
            assert_eq!(ids, ["e"], "{first}");
            assert_eq!(code, Some(3), "{first}");
        }
        None => assert_eq!(code, Some(0), "{first}"),
    }

    let (code, second) = json("2");
    assert_eq!(code, Some(3), "{second}");
    assert_eq!(second["events"].as_array().unwrap().len(), 1, "{second}");
    assert_eq!(second["errors"].as_array().unwrap().len(), 1, "{second}");
    assert_eq!(second["errors"][0]["id"], "d", "{second}");
    assert_eq!(second["nextPage"], Value::Null, "{second}");
}

/// The agenda's JSON lists its events under `events`, each with its item
/// id, its start as `event list --json` prints it and its label, beside
/// the items it could not read under `errors`.
#[test]
fn the_agenda_json_holds_its_events_and_its_errors() {
    let home = account();
    let args = ["--json", "event", "agenda", "11", "2026"];
    let json: Value = serde_json::from_str(&run(home.path(), &args)).unwrap();

    let events = json["events"].as_array().unwrap();
    assert_eq!(events.len(), 3, "{json}");
    let first = serde_json::json!({
        "id": "a",
        "start": "2026-11-12T09:00:00Z",
        "label": "Standup",
    });
    assert_eq!(events[0], first, "{json}");
    assert_eq!(json["errors"], Value::Array(Vec::new()), "{json}");

    let (home, _) = unreadable_account();
    let output = output(home.path(), &args);
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(3), "{json}");
    let last = json["errors"].as_array().unwrap().last().unwrap().clone();
    assert_eq!(last["id"], "d", "{json}");
    assert_eq!(last["code"], "event-unreadable", "{json}");
}

/// The agenda stamps a UTC event at its local time, as `event list` shows
/// it, its JSON `start` the instant as `event list --json` prints it.
#[test]
fn the_agenda_stamps_an_event_in_the_local_zone() {
    let home = account();
    let config = home.path().join("config.toml");
    let agenda = |tz: &str, json: bool| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_calendula"));
        command.args(["-c", config.to_str().unwrap()]);
        if json {
            command.arg("--json");
        }
        let output = command
            .args(["event", "agenda", "11", "2026"])
            .env("TZ", tz)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        String::from_utf8(output.stdout).unwrap()
    };

    let json: Value = serde_json::from_str(&agenda("Europe/Paris", true)).unwrap();
    assert_eq!(json["events"][0]["start"], "2026-11-12T09:00:00Z", "{json}");
    assert_eq!(json["events"][0]["label"], "Standup", "{json}");

    let text = agenda("America/New_York", false);
    assert!(text.contains("Nov 12, 04:00"), "{text}");

    let listed = output(home.path(), &["event", "list"]);
    let listed = String::from_utf8(listed.stdout).unwrap();
    assert!(listed.contains("2026-11-12 09:00"), "{listed}");
}

/// CAL-12: a window pages its occurrences in start order across the whole
/// window, whatever order the backend lists its items in.
#[test]
fn a_window_pages_its_events_in_start_order() {
    let home = account();
    let calendar = home.path().join("personal");
    event(
        &calendar,
        "z",
        "20261111T080000Z",
        "20261111T083000Z",
        "Earliest",
    );

    let page = |page: &str| {
        let args = [
            "--json",
            "event",
            "list",
            "--from",
            "2026-11-10",
            "--to",
            "2026-11-20",
            "-s",
            "2",
            "-p",
            page,
        ];
        let json: Value = serde_json::from_str(&run(home.path(), &args)).unwrap();
        let summaries: Vec<String> = json["events"]
            .as_array()
            .unwrap()
            .iter()
            .map(|event| event["summary"].as_str().unwrap_or("").to_owned())
            .collect();
        (summaries, json["nextPage"].clone())
    };

    assert_eq!(
        page("1"),
        (vec!["Earliest".into(), "Standup".into()], "2".into())
    );
    assert_eq!(
        page("2"),
        (vec!["Review".into(), String::new()], Value::Null)
    );
}
