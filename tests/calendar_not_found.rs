//! # Calendar not found
//!
//! The built binary naming a calendar its vdir home does not hold: every
//! command fails coded `calendar-not-found` and exits 1, reading, creating
//! and removing nothing, a path reaching out of the home included. A new
//! calendar id that is not one directory name is refused, writing nothing.

#![cfg(feature = "vdir")]

use std::{
    fs,
    path::Path,
    process::{Command, Output, Stdio},
};

use serde_json::Value;
use tempfile::TempDir;

/// A directory holding `outer`, which holds the vdir home `home`, its
/// calendar `personal` with the one event `a`, and the directory `keep`
/// beside the home with one file; and the configuration file.
fn account() -> TempDir {
    let dir = TempDir::new().unwrap();
    let outer = dir.path().join("outer");
    let calendar = outer.join("home").join("personal");
    fs::create_dir_all(&calendar).unwrap();
    fs::create_dir(outer.join("keep")).unwrap();
    fs::write(outer.join("keep").join("file"), "kept").unwrap();

    let ics = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\n\
        BEGIN:VEVENT\r\nUID:a@example.org\r\nDTSTAMP:20261101T000000Z\r\n\
        DTSTART:20261112T090000Z\r\nSUMMARY:Standup\r\nEND:VEVENT\r\n\
        END:VCALENDAR\r\n";
    fs::write(calendar.join("a.ics"), ics).unwrap();

    let config = format!(
        "[accounts.perso]\ndefault = true\nvdir.root = {:?}\n",
        outer.join("home").display().to_string()
    );
    fs::write(dir.path().join("config.toml"), config).unwrap();

    dir
}

/// Runs the binary with `args` over the account in `dir`.
fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_calendula"))
        .arg("-c")
        .arg(dir.join("config.toml"))
        .args(args)
        .env("HOME", dir)
        .env("XDG_CONFIG_HOME", dir)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

/// The failure object `out` printed, once it exited 1.
fn failure(out: &Output) -> Value {
    let stdout = String::from_utf8_lossy(&out.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap_or_else(|_| panic!("{stdout}"));

    assert_eq!(out.status.code(), Some(1), "{json}");
    json
}

/// Fails unless `out` is the failure object coded `calendar-not-found`
/// saying `error`, exiting 1.
fn refused(out: &Output, error: &str) {
    let json = failure(out);

    assert_eq!(json["code"], "calendar-not-found", "{json}");
    assert_eq!(json["error"], error, "{json}");
}

/// Fails unless the file beside the home is still there.
fn kept(dir: &Path) {
    let file = dir.join("outer").join("keep").join("file");
    assert_eq!(fs::read_to_string(&file).unwrap(), "kept");
}

/// The paths under `dir`, sorted, relative to it.
fn tree(dir: &Path) -> Vec<String> {
    let mut paths = Vec::new();
    let mut pending = vec![dir.to_path_buf()];

    while let Some(path) = pending.pop() {
        for entry in fs::read_dir(&path).unwrap() {
            let path = entry.unwrap().path();
            let relative = path.strip_prefix(dir).unwrap();
            paths.push(relative.display().to_string());

            if path.is_dir() {
                pending.push(path);
            }
        }
    }

    paths.sort();
    paths
}

#[test]
fn every_unified_command_refuses_an_unknown_calendar() {
    let dir = account();
    let before = tree(dir.path());

    let commands: &[&[&str]] = &[
        &["event", "list", "-k", "nope"],
        &["item", "list", "-k", "nope"],
        &["event", "read", "-k", "nope", "a"],
        &["item", "read", "-k", "nope", "a"],
        &["event", "add", "-k", "nope", "--start", "2026-11-20T10:00"],
        &["event", "update", "-k", "nope", "a", "--summary", "Review"],
        &["event", "delete", "-k", "nope", "a"],
        &["item", "delete", "-k", "nope", "a"],
        &["calendar", "update", "nope", "-n", "Renamed"],
        &["calendar", "delete", "nope"],
    ];

    for args in commands {
        let out = run(dir.path(), &[&["--json"], *args].concat());
        refused(&out, "Calendar `nope` not found");
    }

    assert_eq!(tree(dir.path()), before);
}

#[test]
fn a_calendar_reaching_out_of_the_home_is_not_found() {
    let dir = account();
    let before = tree(dir.path());

    let ids = [
        "",
        ".",
        "..",
        "../keep",
        "personal/..",
        "personal/",
        "personal/.",
    ];

    for id in ids {
        let out = run(dir.path(), &["--json", "calendar", "delete", "-k", id]);
        refused(&out, &format!("Calendar `{id}` not found"));
        kept(dir.path());

        let out = run(dir.path(), &["--json", "event", "list", "-k", id]);
        refused(&out, &format!("Calendar `{id}` not found"));
    }

    assert_eq!(tree(dir.path()), before);
}

#[test]
fn the_vdir_commands_refuse_an_unknown_collection() {
    let dir = account();
    let before = tree(dir.path());

    let commands: &[&[&str]] = &[
        &["vdir", "delete", "nope"],
        &["vdir", "delete", ".."],
        &["vdir", "delete", "../keep"],
        &["vdir", "delete", "personal/"],
        &["vdir", "delete", "personal/."],
        &["vdir", "rename", "nope", "renamed"],
        &["vdir", "rename", "personal/", "renamed"],
    ];

    for args in commands {
        let out = run(dir.path(), &[&["--json"], *args].concat());
        refused(&out, &format!("Collection `{}` not found", args[2]));
        kept(dir.path());
    }

    assert_eq!(tree(dir.path()), before);
}

#[test]
fn a_new_id_reaching_out_of_the_home_is_refused() {
    let dir = account();
    let before = tree(dir.path());

    let commands: &[&[&str]] = &[
        &["calendar", "create", "..", "Escape"],
        &["calendar", "create", "../keep", "Escape"],
        &["calendar", "create", "../escape", "Escape"],
        &["calendar", "create", "x/y", "Escape"],
        &["vdir", "create", ".."],
        &["vdir", "create", "../keep"],
        &["vdir", "create", "../escape"],
        &["vdir", "create", "x/y"],
        &["vdir", "rename", "personal", ".."],
        &["vdir", "rename", "personal", "../escape"],
    ];

    for args in commands {
        let json = failure(&run(dir.path(), &[&["--json"], *args].concat()));
        assert_eq!(json["code"], Value::Null, "{args:?}: {json}");
        kept(dir.path());
    }

    assert_eq!(tree(dir.path()), before);
}

#[test]
fn a_calendar_the_home_holds_is_read() {
    let dir = account();
    let out = run(dir.path(), &["--json", "event", "list", "-k", "personal"]);
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();

    assert_eq!(out.status.code(), Some(0), "{json}");
    assert_eq!(json["events"][0]["id"], "a", "{json}");
}
