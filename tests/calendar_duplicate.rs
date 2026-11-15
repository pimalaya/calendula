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
            &["calendar", "create", "work", "Other"],
            "Calendar `work` already exists",
        ),
        (
            &["vdir", "create", "work", "-n", "Other"],
            "Collection `work` already exists",
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
    let out = run(dir.path(), &["calendar", "create", "home", "Home"]);
    assert_eq!(out.status.code(), Some(0));
}
