//! # Pimdir search
//!
//! The built binary searching an indexed pimdir store page by page: each
//! page names the next one, which `--page-token` takes back, the last
//! names none, and a token that is no page number is refused.

#![cfg(feature = "pimdir")]

use std::{
    fs,
    io::Write,
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
/// naming the budget, synced and indexed, their summaries in date order.
fn account() -> TempDir {
    account_of(&[
        ("a", "20261110T090000Z"),
        ("b", "20261111T090000Z"),
        ("c", "20261112T090000Z"),
    ])
}

/// A pimdir account over a store whose calendar `cal` holds an event
/// `Budget <uid>` per `(uid, start)` of `starts`, synced and indexed.
fn account_of(starts: &[(&str, &str)]) -> TempDir {
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
        .unwrap()
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

    for (uid, start) in starts {
        json(
            &run(home.path(), &["--json", "event", "add", &event(uid, start)]),
            0,
        );
    }

    let mut owner = PimdirStore::open(&root)
        .unwrap()
        .for_source("caldav")
        .unwrap();
    assert_eq!(owner.drain().unwrap().applied, starts.len());
    drop(owner);

    indexed(&root);
    home
}

/// Refreshes the index of the store at `root` until a pass folds it.
///
/// A pass that finds index.lock held does nothing and says so (`busy`),
/// for its owner to run again: a child another test forks keeps a copy
/// of the lock `PimdirIndex::open` just released until it executes.
fn indexed(root: &Path) {
    let mut index = PimdirIndex::open(root).unwrap();
    for _ in 0..100 {
        let report = index
            .refresh(&PimdirIndexRefreshOptions::default())
            .unwrap();
        if !report.busy {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("index.lock of {} stayed held", root.display());
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
    let next = format!("\nNext page: --page-token {token}\n");
    assert!(text.contains(&next), "{text}");
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
        assert_eq!(error["code"], "page-token-invalid", "{error}");
        assert!(message.contains("Invalid page token"), "{error}");
    }
}

/// `event search` takes the query `pimdir search` takes and answers the
/// same events as `event list` rows, paged by the index's cursor; a query
/// the language refuses is `query-refused`.
#[test]
fn an_event_search_answers_the_query_as_event_rows() {
    let home = account();
    let home = home.path();

    let first = json(
        &run(home, &["--json", "event", "search", "-s", "2", "budget"]),
        0,
    );
    let events = first["events"].as_array().unwrap();
    assert_eq!(events.len(), 2, "{first}");
    assert_eq!(events[0]["calendarId"], "cal", "{first}");
    assert_eq!(first["complete"], true, "{first}");
    let token = first["nextPage"].as_str().unwrap().to_owned();

    let args = ["--json", "pimdir", "search", "-s", "2", "budget"];
    let hits = json(&run(home, &args), 0);
    assert_eq!(hits["nextPage"].as_str(), Some(token.as_str()), "{hits}");

    let args = [
        "--json",
        "event",
        "search",
        "-s",
        "2",
        "--page-token",
        &token,
        "budget",
    ];
    let last = json(&run(home, &args), 0);
    assert_eq!(last["events"].as_array().unwrap().len(), 1, "{last}");
    assert_eq!(last["nextPage"], Value::Null, "{last}");

    let refused = json(&run(home, &["--json", "event", "search", "summary:"]), 1);
    assert_eq!(refused["code"], "query-refused", "{refused}");
}

/// The summaries `args` lists, in its order, from the `hits` of `pimdir
/// search` or the `events` of `event search`.
fn summaries(home: &Path, args: &[&str]) -> Vec<String> {
    let out = json(&run(home, args), 0);
    let rows = out.get("hits").or(out.get("events")).unwrap();
    rows.as_array()
        .unwrap()
        .iter()
        .map(|row| row["summary"].as_str().unwrap().to_owned())
        .collect()
}

/// `--sort` takes pimdir's order names, `newest-first` by default, and
/// reads the former `date` and `name` as `newest-first` and `title`, on
/// `event search` and `pimdir search` alike.
#[test]
fn a_search_sorts_by_the_pimdir_order_names() {
    let home = account_of(&[
        ("b", "20261110T090000Z"),
        ("c", "20261111T090000Z"),
        ("a", "20261112T090000Z"),
    ]);
    let home = home.path();

    for command in ["event", "pimdir"] {
        let sorted = |sort: Option<&str>| {
            let mut args = vec!["--json", command, "search"];
            if let Some(sort) = sort {
                args.extend(["--sort", sort]);
            }
            args.push("budget");
            summaries(home, &args)
        };

        let newest = sorted(None);
        assert_eq!(newest, ["Budget a", "Budget c", "Budget b"], "{command}");
        assert_eq!(sorted(Some("newest-first")), newest, "{command}");
        assert_eq!(sorted(Some("date")), newest, "{command}");

        let mut oldest = newest.clone();
        oldest.reverse();
        assert_eq!(sorted(Some("oldest-first")), oldest, "{command}");

        let title = sorted(Some("title"));
        assert_eq!(title, ["Budget a", "Budget b", "Budget c"], "{command}");
        assert_eq!(sorted(Some("name")), title, "{command}");

        assert_eq!(sorted(Some("relevance")).len(), 3, "{command}");

        let help = run(home, &[command, "search", "--help"]);
        let help = String::from_utf8(help.stdout).unwrap();
        for value in ["newest-first", "oldest-first", "title", "relevance"] {
            assert!(help.contains(value), "{command}: {help}");
        }
        assert!(!help.contains("sender"), "{command}: {help}");
    }
}

/// The MCP tool `event_search` lists pimdir's order names as the values
/// of its `sort`, and none of the former names.
#[test]
fn the_event_search_tool_lists_the_pimdir_order_names() {
    let home = account();
    let home = home.path();

    let mut child = Command::new(env!("CARGO_BIN_EXE_calendula"))
        .arg("-c")
        .arg(home.join("config.toml"))
        .arg("mcp")
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .env_remove("XDG_CONFIG_DIRS")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let list = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#;
    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "{list}").unwrap();
    drop(stdin);
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let response: Value = serde_json::from_slice(&out.stdout).unwrap();
    let tools = response["result"]["tools"].as_array().unwrap();
    let search = tools.iter().find(|tool| tool["name"] == "event_search");
    let search = search.unwrap_or_else(|| panic!("{response}"));
    let sort = &search["inputSchema"]["properties"]["sort"];
    let values = ["newest-first", "oldest-first", "title", "relevance"];
    assert_eq!(sort["enum"], serde_json::json!(values), "{sort}");
}

/// A query holding a reserved form is refused `query-refused` naming
/// `reserved-form`, while an instant and the new relative units are
/// answered.
#[test]
fn a_reserved_form_is_refused_and_instants_are_answered() {
    let home = account();
    let home = home.path();

    for query in ["~x", "/re/", "summary:{x}"] {
        let refused = json(&run(home, &["--json", "event", "search", query]), 1);
        assert_eq!(refused["code"], "query-refused", "{query}: {refused}");
        let message = refused.to_string();
        assert!(message.contains("reserved-form"), "{query}: {refused}");
    }

    let at = summaries(
        home,
        &["--json", "event", "search", "when:2026-11-11T09:00"],
    );
    assert_eq!(at, ["Budget b"]);

    for query in [
        "when:2026-11-07T10:00..",
        "date:2h..",
        "date:..30min",
        "date:..1mo",
    ] {
        json(&run(home, &["--json", "event", "search", query]), 0);
    }
}
