//! # Calendar duplicate
//!
//! The built binary creating a calendar its vdir home already holds:
//! `calendar create` and `vdir create` fail coded `calendar-duplicate` and
//! exit 1, the calendar's marker files left as they were.

#![cfg(feature = "vdir")]

use std::{
    fs,
    path::Path,
    process::{Command, Output, Stdio},
};

use serde_json::Value;
use tempfile::TempDir;

/// A vdir home holding the calendar `work`, named `Work`, and the
/// configuration file.
fn account() -> TempDir {
    let dir = TempDir::new().unwrap();
    let calendar = dir.path().join("home").join("work");
    fs::create_dir_all(&calendar).unwrap();
    fs::write(calendar.join("displayname"), "Work").unwrap();

    let config = format!(
        "[accounts.perso]\ndefault = true\nvdir.root = {:?}\n",
        dir.path().join("home").display().to_string()
    );
    fs::write(dir.path().join("config.toml"), config).unwrap();

    dir
}

/// Runs the binary with `args` over the account in `dir`, under `--json`.
fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_calendula"))
        .arg("-c")
        .arg(dir.join("config.toml"))
        .arg("--json")
        .args(args)
        .env("HOME", dir)
        .env("XDG_CONFIG_HOME", dir)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

#[test]
fn a_calendar_created_twice_is_a_duplicate() {
    let dir = account();

    let commands: &[(&[&str], &str)] = &[
        (
            &["calendar", "create", "Other", "--id", "work"],
            "Cannot create calendar `work`: it exists already",
        ),
        (
            &["vdir", "create", "work", "-n", "Other"],
            "Cannot create calendar `work`: it exists already",
        ),
    ];

    for (args, error) in commands {
        let out = run(dir.path(), args);
        let stdout = String::from_utf8_lossy(&out.stdout);
        let json: Value = serde_json::from_str(&stdout).unwrap();

        assert_eq!(out.status.code(), Some(1), "{args:?}: {stdout}");
        assert_eq!(json["code"], "calendar-duplicate", "{args:?}: {json}");
        assert_eq!(json["error"], *error, "{args:?}: {json}");

        let name = fs::read_to_string(dir.path().join("home/work/displayname")).unwrap();
        assert_eq!(name, "Work", "{args:?}");
    }

    // NOTE: a new id still makes its calendar.
    let out = run(dir.path(), &["calendar", "create", "Home", "--id", "home"]);
    assert_eq!(out.status.code(), Some(0));
}

/// A rename onto a calendar the home holds, a full one or an empty
/// directory, is refused before anything moves.
#[test]
fn a_calendar_renamed_onto_another_is_a_duplicate() {
    let dir = account();
    let home = dir.path().join("home");
    let perso = home.join("perso");
    fs::create_dir_all(&perso).unwrap();
    fs::write(perso.join("x.ics"), "BEGIN:VCALENDAR\r\nEND:VCALENDAR\r\n").unwrap();
    fs::create_dir_all(home.join("empty")).unwrap();

    for target in ["perso", "empty"] {
        let output = run(dir.path(), &["vdir", "rename", "work", target]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(output.status.code(), Some(1), "{target}: {stdout}");

        let json: Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(json["code"], "calendar-duplicate", "{json}");
        assert_eq!(
            json["error"],
            format!("Cannot rename calendar `work` to `{target}`: it exists already"),
            "{json}"
        );

        let name = fs::read_to_string(home.join("work").join("displayname")).unwrap();
        assert_eq!(name, "Work");
        assert!(home.join(target).is_dir(), "{target}");
    }

    assert!(perso.join("x.ics").exists());
}

/// CAR-1: `vdir item move` and `copy` onto an item of the same id fail
/// coded `item-duplicate`, both items left as they were.
#[test]
fn an_item_moved_or_copied_onto_its_id_is_a_duplicate() {
    let dir = account();
    let home = dir.path().join("home");
    fs::create_dir_all(home.join("home")).unwrap();
    fs::write(home.join("work").join("e1.ics"), "work copy").unwrap();
    fs::write(home.join("home").join("e1.ics"), "home copy").unwrap();

    for verb in ["move", "copy"] {
        let output = run(dir.path(), &["vdir", "item", verb, "home", "work", "e1"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(output.status.code(), Some(1), "{verb}: {stdout}");

        let json: Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(json["code"], "item-duplicate", "{json}");

        let work = fs::read_to_string(home.join("work").join("e1.ics")).unwrap();
        let other = fs::read_to_string(home.join("home").join("e1.ics")).unwrap();
        assert_eq!((work.as_str(), other.as_str()), ("work copy", "home copy"));
    }
}
