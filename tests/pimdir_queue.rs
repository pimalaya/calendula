//! # Pimdir queue
//!
//! The built binary over a pimdir store: every pending row of its queue
//! lists whatever its kind, a row shows as it lists, and the refusals
//! carry the codes cardamum's carry, an existing calendar's creation a
//! duplicate before a missing capability.

#![cfg(feature = "pimdir")]

use std::{
    fs,
    path::Path,
    process::{Command, Output, Stdio},
};

use io_pimdir::{
    change::PimdirWriteOp,
    client::PimdirStore,
    collection::{PimdirCheckpoint, PimdirCollectionId, PimdirScope},
};
use serde_json::Value;
use tempfile::TempDir;

const EVENT: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\nBEGIN:VEVENT\r\n\
    UID:standup@example.org\r\nDTSTAMP:20261101T000000Z\r\nDTSTART:20261110T090000Z\r\n\
    SUMMARY:Standup\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

/// Runs the binary with `args` under `--json` over the account in
/// `home`, its stdin closed.
fn run(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_calendula"))
        .arg("-c")
        .arg(home.join("config.toml"))
        .arg("--json")
        .args(args)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .env("TZ", "UTC")
        .env_remove("XDG_CONFIG_DIRS")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

/// The JSON a run printed, failing unless it exited with `status`.
fn json(home: &Path, args: &[&str], status: i32) -> Value {
    let out = run(home, args);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(status), "{args:?}: {stdout}");
    serde_json::from_slice(&out.stdout).unwrap_or_else(|err| panic!("{err}: {stdout}"))
}

/// A pimdir account over a store whose calendar `cal` a source synced,
/// holding the event `Standup` and a pending removal of it.
fn account() -> TempDir {
    let home = TempDir::new().unwrap();
    let root = home.path().join("store");
    let config = format!(
        "[accounts.perso]\ndefault = true\ncalendar.default = \"cal\"\npimdir.root = {:?}\n",
        root.display().to_string(),
    );
    fs::write(home.path().join("config.toml"), config).unwrap();

    let store = PimdirStore::open(&root).unwrap();
    store.ensure_collection("cal", "text/calendar").unwrap();
    store
        .for_source("caldav")
        .write(vec![
            PimdirWriteOp::OpenRound {
                collection: PimdirCollectionId("cal".into()),
                scope: PimdirScope::unbounded(),
                band: false,
            },
            PimdirWriteOp::CloseRound {
                collection: PimdirCollectionId("cal".into()),
                coverage: PimdirScope::unbounded(),
                checkpoint: Some(PimdirCheckpoint(b"0".to_vec())),
            },
        ])
        .unwrap();

    json(home.path(), &["event", "add", EVENT], 0);
    let mut owner = PimdirStore::open(&root).unwrap().for_source("caldav");
    assert_eq!(owner.drain().unwrap().applied, 1);
    drop(owner);

    let events = json(home.path(), &["event", "list"], 0);
    let id = events["events"][0]["id"].as_str().unwrap().to_owned();
    json(home.path(), &["event", "delete", &id], 0);

    home
}

/// A removal lists among the pending rows, not only creations and
/// intents, and `queue show` prints the row it lists plus its state.
#[test]
fn every_pending_row_lists_and_shows_as_listed() {
    let home = account();
    let home = home.path();

    let queue = json(home, &["pimdir", "queue", "list"], 0);
    let actions = queue["actions"].as_array().unwrap();
    assert_eq!(actions.len(), 1, "{queue}");
    assert_eq!(actions[0]["kind"], "remove", "{queue}");
    assert_eq!(queue["errors"], Value::Array(Vec::new()), "{queue}");

    let row = actions[0]["row"].to_string();
    let mut shown = json(home, &["pimdir", "queue", "show", &row], 0);
    assert_eq!(shown["state"], "pending", "{shown}");

    let shown = shown.as_object_mut().unwrap();
    for read in ["state", "appliedAt", "refusals"] {
        shown.remove(read);
    }
    assert_eq!(&Value::Object(shown.clone()), &actions[0], "{queue}");
}

/// A row the queue does not hold, and a cancellation no terminal can
/// confirm, are refused with the codes cardamum refuses them with.
#[test]
fn the_queue_refusals_carry_their_codes() {
    let home = account();
    let home = home.path();

    let cases: [(&[&str], &str); 3] = [
        (&["pimdir", "queue", "show", "999"], "row-not-found"),
        (
            &["pimdir", "queue", "cancel", "999", "--yes"],
            "row-not-found",
        ),
        (&["pimdir", "queue", "cancel", "999"], "row-not-found"),
    ];
    for (args, code) in cases {
        let err = json(home, args, 1);
        assert_eq!(err["code"], code, "{args:?}: {err}");
    }

    let queue = json(home, &["pimdir", "queue", "list"], 0);
    let row = queue["actions"][0]["row"].to_string();
    let err = json(home, &["pimdir", "queue", "cancel", &row], 1);
    assert_eq!(err["code"], "confirmation-required", "{err}");

    let queue = json(home, &["pimdir", "queue", "list"], 0);
    assert_eq!(queue["actions"].as_array().unwrap().len(), 1, "{queue}");
}

/// A create naming a calendar the store holds fails as a duplicate,
/// before the refusal of a create the store takes from no producer.
#[test]
fn an_existing_calendar_is_a_duplicate_before_a_missing_capability() {
    let home = account();
    let home = home.path();

    let err = json(home, &["calendar", "create", "Other", "--id", "cal"], 1);
    assert_eq!(err["code"], "calendar-duplicate", "{err}");
    let err = json(home, &["calendar", "create", "cal"], 1);
    assert_eq!(err["code"], "calendar-duplicate", "{err}");

    let err = json(home, &["calendar", "create", "Other"], 1);
    assert_eq!(err["code"], "capability-missing", "{err}");
}
