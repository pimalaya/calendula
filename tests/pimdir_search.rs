//! # Pimdir search
//!
//! The built binary searching an indexed pimdir store page by page: each
//! page names the next one, which `--page-token` takes back, the last
//! names none, and a token that is no page number is refused.

#![cfg(feature = "pimdir")]

use std::{
    fs,
    path::Path,
    process::{Command, Output, Stdio},
};

use io_pimdir::{
    change::PimdirWriteOp,
    client::{
        PimdirStore,
        search::{PimdirIndex, PimdirIndexRefreshOptions},
    },
    collection::{PimdirCheckpoint, PimdirCollectionId, PimdirScope},
};
use serde_json::Value;
use tempfile::TempDir;

/// One event `uid` at `start`, its summary naming the budget.
fn event(uid: &str, start: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\nBEGIN:VEVENT\r\n\
         UID:{uid}@example.org\r\nDTSTAMP:20261101T000000Z\r\nDTSTART:{start}\r\n\
         SUMMARY:Budget {uid}\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
    )
}

/// Runs the binary with `args` over the account in `home`, in UTC.
fn run(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_calendula"))
        .arg("-c")
        .arg(home.join("config.toml"))
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
fn json(out: &Output, status: i32) -> Value {
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(status), "{stdout}");
    serde_json::from_slice(&out.stdout).unwrap_or_else(|err| panic!("{err}: {stdout}"))
}

/// A pimdir account over a store whose calendar `cal` holds three events
/// naming the budget, synced and indexed.
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

    let starts = [
        ("a", "20261110T090000Z"),
        ("b", "20261111T090000Z"),
        ("c", "20261112T090000Z"),
    ];
    for (uid, start) in starts {
        json(
            &run(home.path(), &["--json", "event", "add", &event(uid, start)]),
            0,
        );
    }

    let mut owner = PimdirStore::open(&root).unwrap().for_source("caldav");
    assert_eq!(owner.drain().unwrap().applied, 3);
    drop(owner);

    PimdirIndex::open(&root)
        .unwrap()
        .refresh(&PimdirIndexRefreshOptions::default())
        .unwrap();

    home
}

#[test]
fn a_search_resumes_at_the_next_page_it_names() {
    let home = account();
    let home = home.path();

    let search = |extra: &[&str]| {
        let args = [
            &["--json", "pimdir", "search", "-s", "2"],
            extra,
            &["budget"],
        ]
        .concat();
        json(&run(home, &args), 0)
    };

    let first = search(&[]);
    assert_eq!(first["hits"].as_array().unwrap().len(), 2, "{first}");
    let token = first["nextPage"].as_str().unwrap().to_owned();
    assert_eq!(first["errors"], Value::Array(Vec::new()), "{first}");
    assert!(first.get("more").is_none(), "{first}");

    let last = search(&["--page-token", &token]);
    assert_eq!(last["hits"].as_array().unwrap().len(), 1, "{last}");
    assert_eq!(last["nextPage"], Value::Null, "{last}");

    let numbered = search(&["--page", "2"]);
    assert_eq!(numbered["hits"], last["hits"], "{numbered}");

    let past = search(&["--page", "3"]);
    assert_eq!(past["hits"], Value::Array(Vec::new()), "{past}");
    assert_eq!(past["nextPage"], Value::Null, "{past}");

    let won = search(&["--page", "1", "--page-token", &token]);
    assert_eq!(won["hits"], last["hits"], "{won}");

    let whole = json(
        &run(home, &["--json", "pimdir", "search", "-s", "0", "budget"]),
        0,
    );
    assert_eq!(
        whole["hits"].as_array().unwrap().len(),
        3,
        "CAL-18: {whole}"
    );
    assert_eq!(whole["nextPage"], Value::Null, "{whole}");

    let text = run(home, &["pimdir", "search", "-s", "2", "budget"]);
    let text = String::from_utf8(text.stdout).unwrap();
    assert!(text.contains("Page:      1, events 1 to 2\n"), "{text}");
    assert!(text.contains("\nNext page: --page 2\n"), "{text}");
}

#[test]
fn an_unreadable_page_token_is_refused() {
    let home = account();

    for token in ["0", "two"] {
        let args = [
            "--json",
            "pimdir",
            "search",
            "--page-token",
            token,
            "budget",
        ];
        let error = json(&run(home.path(), &args), 1);
        let message = error["error"].as_str().unwrap_or_default();
        assert!(message.contains("Invalid page token"), "{error}");
    }
}
