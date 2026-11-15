//! # Write
//!
//! The built binary writing to a vdir account: every receipt says whether
//! a sync applies the write later, `calendar create` names the calendar it
//! made, a command naming no calendar where none is the default fails
//! coded, and a build written to a file names its size.

#![cfg(feature = "vdir")]

use std::{
    fs,
    path::Path,
    process::{Command, Output, Stdio},
};

use serde_json::Value;
use tempfile::TempDir;

const EVENT: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\n\
                     BEGIN:VEVENT\r\nUID:standup@example.org\r\n\
                     DTSTAMP:20261101T000000Z\r\nDTSTART:20261112T090000Z\r\n\
                     SUMMARY:Standup\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

/// A vdir home holding the empty calendar `personal`, and the
/// configuration of an account reading it, `default` its
/// `calendar.default` line.
fn account(default: &str) -> TempDir {
    let home = TempDir::new().unwrap();
    fs::create_dir(home.path().join("personal")).unwrap();

    let config = format!(
        "[accounts.perso]\ndefault = true\n{default}vdir.home-dir = {:?}\n",
        home.path().display().to_string()
    );
    fs::write(home.path().join("config.toml"), config).unwrap();

    home
}

/// Runs the binary with `args` over the account in `home`.
fn run(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_calendula"))
        .arg("-c")
        .arg(home.join("config.toml"))
        .args(args)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .env_remove("XDG_CONFIG_DIRS")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

/// The JSON a run printed, failing unless it exited with `status`.
fn json(out: &Output, status: i32) -> Value {
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(status), "{stdout}");
    serde_json::from_slice(&out.stdout).unwrap_or_else(|err| panic!("{err}: {stdout}"))
}

#[test]
fn every_write_receipt_says_whether_it_is_queued() {
    let home = account("calendar.default = \"personal\"\n");
    let home = home.path();

    let created = json(&run(home, &["--json", "event", "add", EVENT]), 0);
    assert_eq!(created["queued"], false, "{created}");
    let id = created["id"].as_str().unwrap().to_owned();

    let event = EVENT.replace("Standup", "Retro");
    let updated = json(&run(home, &["--json", "event", "update", &id, &event]), 0);
    assert_eq!(updated, serde_json::json!({ "id": id, "queued": false }));

    let item = EVENT.replace("standup@", "item@");
    let created = json(&run(home, &["--json", "item", "add", &item]), 0);
    assert_eq!(created["queued"], false, "{created}");
    let id = created["id"].as_str().unwrap().to_owned();

    let updated = json(&run(home, &["--json", "item", "update", &id, &item]), 0);
    assert_eq!(updated, serde_json::json!({ "id": id, "queued": false }));

    let args = ["--json", "calendar", "create", "work", "Work"];
    let created = json(&run(home, &args), 0);
    let expected = serde_json::json!({ "id": "work", "name": "Work", "queued": false });
    assert_eq!(created, expected);
}

#[test]
fn no_calendar_and_no_default_one_is_coded() {
    let home = account("");

    for args in [
        &["--json", "event", "list"][..],
        &["--json", "item", "read", "a"],
        &["--json", "event", "add", EVENT],
    ] {
        let error = json(&run(home.path(), args), 1);
        assert_eq!(
            error["code"], "calendar-default-missing",
            "{args:?}: {error}"
        );

        let message = error["error"].as_str().unwrap_or_default();
        assert!(message.contains("`-k <CALENDAR-ID>`"), "{message}");
        assert!(message.contains("`calendar.default`"), "{message}");
    }
}

#[test]
fn a_build_written_to_a_file_names_its_size() {
    let home = account("");
    let path = home.path().join("built.ics");
    let target = path.to_str().unwrap();

    let out = run(home.path(), &["item", "build", "-o", target, EVENT]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "{stdout}");

    let size = fs::metadata(&path).unwrap().len();
    assert_eq!(stdout, format!("Saved {size} bytes to {target}\n"));

    let saved = json(
        &run(
            home.path(),
            &["--json", "item", "build", "-o", target, EVENT],
        ),
        0,
    );
    assert_eq!(saved, serde_json::json!({ "path": target, "size": size }));
}

/// `calendar create` takes a name alone, the vdir directory named after
/// it, and an id before the name as before; `calendar update` and
/// `calendar delete` take the calendar as an argument, the former `-k`
/// still read; `-C` sets the color.
#[test]
fn the_calendar_commands_take_the_calendar_as_an_argument() {
    let home = account("");
    let home = home.path();

    let created = json(&run(home, &["--json", "calendar", "create", "Work"]), 0);
    let expected = serde_json::json!({ "id": "Work", "name": "Work", "queued": false });
    assert_eq!(created, expected);
    assert!(home.join("Work").is_dir());

    let args = [
        "--json", "calendar", "create", "team", "The team", "-C", "#00ff00",
    ];
    let created = json(&run(home, &args), 0);
    assert_eq!(created["id"], "team", "{created}");
    let color = fs::read_to_string(home.join("team").join("color")).unwrap();
    assert_eq!(color.trim(), "#00ff00");

    let args = ["--json", "calendar", "create", "x/y"];
    let error = json(&run(home, &args), 1);
    let message = error["error"].as_str().unwrap_or_default();
    assert!(message.contains("give the calendar an id"), "{message}");

    let args = [
        "--json", "calendar", "update", "Work", "-n", "Job", "-C", "#ff0000",
    ];
    let updated = json(&run(home, &args), 0);
    assert_eq!(
        updated,
        serde_json::json!({ "id": "Work", "queued": false })
    );
    let name = fs::read_to_string(home.join("Work").join("displayname")).unwrap();
    assert_eq!(name.trim(), "Job");

    let args = ["--json", "calendar", "update", "-k", "team", "-n", "Crew"];
    json(&run(home, &args), 0);

    json(&run(home, &["--json", "calendar", "delete", "Work"]), 0);
    assert!(!home.join("Work").exists());
    json(
        &run(home, &["--json", "calendar", "delete", "-k", "team"]),
        0,
    );
    assert!(!home.join("team").exists());

    let error = json(&run(home, &["--json", "calendar", "delete"]), 2);
    assert_eq!(error["code"], "usage-invalid", "{error}");
}

/// `create` stays a hidden alias of `event add` and `item add`.
#[test]
fn create_still_adds() {
    let home = account("calendar.default = \"personal\"\n");
    let home = home.path();

    let created = json(&run(home, &["--json", "event", "create", EVENT]), 0);
    assert_eq!(created["queued"], false, "{created}");

    let item = EVENT.replace("standup@", "item@");
    let created = json(&run(home, &["--json", "item", "create", &item]), 0);
    assert_eq!(created["queued"], false, "{created}");
}

/// A `--start` naming no zone moves a zoned event on its own clock: it
/// keeps its `TZID`, the end moving with it, rather than turning floating.
#[test]
fn a_local_start_keeps_the_zone_of_the_event() {
    let home = account("calendar.default = \"personal\"\n");
    let home = home.path();

    let zoned = EVENT.replace(
        "DTSTART:20261112T090000Z\r\n",
        "DTSTART;TZID=Europe/Paris:20261112T090000\r\n\
         DTEND;TZID=Europe/Paris:20261112T100000\r\n",
    );
    let created = json(&run(home, &["--json", "event", "add", &zoned]), 0);
    let id = created["id"].as_str().unwrap().to_owned();

    let args = [
        "--json",
        "event",
        "update",
        &id,
        "--start",
        "2026-11-26T11:00",
    ];
    json(&run(home, &args), 0);

    let mut files = fs::read_dir(home.join("personal")).unwrap();
    let file = files.next().unwrap().unwrap().path();
    let stored = fs::read_to_string(file).unwrap();
    assert!(
        stored.contains("DTSTART;TZID=Europe/Paris:20261126T110000\r\n"),
        "{stored}"
    );
    assert!(
        stored.contains("DTEND;TZID=Europe/Paris:20261126T120000\r\n"),
        "{stored}"
    );
}
