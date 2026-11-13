//! # MCP
//!
//! The built binary's `mcp` server over a vdir account: it says how to use
//! its tools, lists the reading ones alone unless `--allow-write`, the
//! scheduling arguments only under `--allow-send` as well, refuses what is
//! no request and goes on, answers a call with the command's JSON, codes a
//! value the command refuses `usage-invalid`, and never writes or deletes
//! an item naming attendees without `--allow-send`.

#![cfg(feature = "vdir")]

use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};

use serde_json::{Value, json};
use tempfile::TempDir;

/// An event of the calendar `personal`, `attendees` its ATTENDEE lines.
fn event(uid: &str, attendees: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\nBEGIN:VEVENT\r\n\
         UID:{uid}@example.org\r\nDTSTAMP:20261101T000000Z\r\n\
         DTSTART:20261112T090000Z\r\nDTEND:20261112T093000Z\r\nSUMMARY:{uid}\r\n\
         {attendees}END:VEVENT\r\nEND:VCALENDAR\r\n"
    )
}

/// The ATTENDEE line of an invited guest.
const GUEST: &str = "ATTENDEE;RSVP=TRUE:mailto:bob@example.org\r\n";

/// A home holding a vdir calendar `personal` with the event `standup`,
/// inviting nobody, and the event `review`, inviting a guest, and the
/// configuration of an account reading it.
fn account() -> TempDir {
    let home = TempDir::new().unwrap();
    let calendar = home.path().join("calendars").join("personal");
    fs::create_dir_all(&calendar).unwrap();
    fs::write(calendar.join("standup.ics"), event("standup", "")).unwrap();
    fs::write(calendar.join("review.ics"), event("review", GUEST)).unwrap();

    let calendars = home.path().join("calendars");
    let toml = format!(
        "[accounts.perso]\ndefault = true\ncalendar.default = \"personal\"\n\
         vdir.home-dir = {:?}\n",
        calendars.display().to_string(),
    );
    fs::write(home.path().join("config.toml"), toml).unwrap();

    home
}

/// The file names of the calendar `personal`, sorted.
fn stored(home: &Path) -> Vec<String> {
    let calendar = home.join("calendars").join("personal");
    let mut names: Vec<String> = fs::read_dir(calendar)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// Runs a server with `flags` in `home` over the bytes of `input`.
fn spawn(home: &Path, flags: &[&str], input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_calendula"))
        .arg("-c")
        .arg(home.join("config.toml"))
        .arg("mcp")
        .args(flags)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .env_remove("XDG_CONFIG_DIRS")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(input).unwrap();
    drop(stdin);

    child.wait_with_output().unwrap()
}

/// Runs a server with `flags` over the bytes of `input`, and returns its
/// responses in order, each one line of stdout.
fn serve_bytes(home: &Path, flags: &[&str], input: &[u8]) -> Vec<Value> {
    let output = spawn(home, flags, input);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");

    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap_or_else(|err| panic!("{err}: {line}")))
        .collect()
}

/// Runs a server with `flags` over `requests`, and returns its responses
/// in order.
fn serve(home: &Path, flags: &[&str], requests: &[Value]) -> Vec<Value> {
    let input: String = requests
        .iter()
        .map(|request| format!("{request}\n"))
        .collect();

    serve_bytes(home, flags, input.as_bytes())
}

fn call(id: u32, name: &str, arguments: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": { "name": name, "arguments": arguments },
    })
}

fn list(id: u32) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "method": "tools/list" })
}

/// The tools a `tools/list` response holds.
fn tools(response: &Value) -> &Vec<Value> {
    response["result"]["tools"].as_array().unwrap()
}

fn names(response: &Value) -> Vec<&str> {
    tools(response)
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect()
}

fn tool<'a>(response: &'a Value, name: &str) -> &'a Value {
    let found = tools(response).iter().find(|tool| tool["name"] == name);
    found.unwrap_or_else(|| panic!("{name} is not listed: {response}"))
}

/// The property names of a tool's input.
fn properties(tool: &Value) -> Vec<&str> {
    let properties = tool["inputSchema"]["properties"].as_object().unwrap();
    properties.keys().map(String::as_str).collect()
}

/// The failure object a failed call carries.
fn report(response: &Value) -> Value {
    let result = &response["result"];
    assert_eq!(result["isError"], true, "{response}");
    let text = result["content"][0]["text"].as_str().unwrap();
    serde_json::from_str(text).unwrap()
}

/// The text of the failure object a failed call carries.
fn refusal(response: &Value) -> String {
    report(response)["error"].as_str().unwrap().to_owned()
}

/// The arguments a tool lists only under `--allow-send`.
const SCHEDULING: [&str; 5] = [
    "attendee",
    "add-attendee",
    "remove-attendee",
    "organizer",
    "send-updates",
];

const READS: [&str; 7] = [
    "account_capability",
    "calendar_list",
    "event_list",
    "event_read",
    "event_find",
    "item_list",
    "item_read",
];

#[test]
fn initialize_tells_the_agent_how_to_use_the_tools() {
    let home = account();
    let responses = serve(
        home.path(),
        &[],
        &[json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "protocolVersion": "2025-06-18" },
        })],
    );

    let result = &responses[0]["result"];
    assert_eq!(result["serverInfo"]["name"], "calendula");
    assert_eq!(result["protocolVersion"], "2025-06-18");

    let instructions = result["instructions"].as_str().unwrap_or_default();
    for word in [
        "calendar_list",
        "event_list",
        "event_find",
        "--allow-write",
        "--allow-send",
        "ATTENDEE",
    ] {
        assert!(instructions.contains(word), "{word}: {instructions}");
    }
}

#[test]
fn reading_tools_only_without_flags() {
    let home = account();
    let responses = serve(home.path(), &[], &[list(1)]);

    assert_eq!(names(&responses[0]), READS);

    for tool in tools(&responses[0]) {
        assert_eq!(tool["inputSchema"]["additionalProperties"], false, "{tool}");
        assert_eq!(tool["annotations"]["readOnlyHint"], true, "{tool}");
        assert_eq!(tool["annotations"]["destructiveHint"], false, "{tool}");
        assert_eq!(tool["annotations"]["openWorldHint"], false, "{tool}");

        if let Some(schema) = tool.get("outputSchema") {
            assert_eq!(schema["type"], "object", "{tool}");
        }
    }
}

#[test]
fn writes_are_listed_under_allow_write_without_scheduling() {
    let home = account();
    let responses = serve(home.path(), &["--allow-write"], &[list(1)]);
    let listed = &responses[0];

    let mut expected = READS.to_vec();
    expected.extend([
        "calendar_create",
        "calendar_update",
        "calendar_delete",
        "event_create",
        "event_update",
        "event_delete",
        "item_create",
        "item_update",
        "item_delete",
    ]);
    let mut names = names(listed);
    names.sort_unstable();
    expected.sort_unstable();
    assert_eq!(names, expected);

    for tool in tools(listed) {
        let properties = properties(tool);
        for gated in SCHEDULING.iter().chain(&["interactive", "composer"]) {
            assert!(!properties.contains(gated), "{gated}: {tool}");
        }
        assert_eq!(tool["annotations"]["openWorldHint"], false, "{tool}");
    }

    assert!(properties(tool(listed, "event_create")).contains(&"summary"));
    assert!(properties(tool(listed, "event_create")).contains(&"ical"));
    assert_eq!(
        tool(listed, "event_create")["annotations"]["readOnlyHint"],
        false
    );

    for name in ["event_update", "event_delete", "item_delete"] {
        assert_eq!(tool(listed, name)["annotations"]["destructiveHint"], true);
    }
    assert_eq!(
        tool(listed, "event_create")["annotations"]["destructiveHint"],
        false
    );
}

#[test]
fn scheduling_is_listed_under_allow_send_as_well() {
    let home = account();
    let flags = ["--allow-write", "--allow-send"];
    let responses = serve(home.path(), &flags, &[list(1)]);
    let listed = &responses[0];

    let create = properties(tool(listed, "event_create"));
    for argument in ["attendee", "add-attendee", "remove-attendee", "organizer"] {
        assert!(create.contains(&argument), "{argument}: {create:?}");
    }
    assert!(properties(tool(listed, "event_delete")).contains(&"send-updates"));

    for name in ["event_create", "event_update", "event_delete"] {
        assert_eq!(tool(listed, name)["annotations"]["openWorldHint"], true);
    }
    for name in ["item_create", "item_update", "item_delete"] {
        assert_eq!(tool(listed, name)["annotations"]["openWorldHint"], true);
    }
    for name in ["calendar_create", "calendar_delete", "event_list"] {
        assert_eq!(tool(listed, name)["annotations"]["openWorldHint"], false);
    }
}

#[test]
fn sending_without_writing_is_refused_at_start() {
    let home = account();
    let output = spawn(home.path(), &["--allow-send"], b"");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(stderr.contains("--allow-write"), "{stderr}");
}

#[test]
fn a_message_that_is_no_request_is_refused_and_the_server_goes_on() {
    let home = account();
    let mut input = Vec::new();
    input.extend_from_slice(br#"[{"jsonrpc":"2.0","id":1,"method":"ping"}]"#);
    input.extend_from_slice(b"\n{\"jsonrpc\":\"2.0\",\"id\":\"\xff\",\"method\":\"ping\"}\n");
    input.extend_from_slice(br#"{"id":3,"method":"ping"}"#);
    input.extend_from_slice(b"\n");
    input.extend_from_slice(br#"{"jsonrpc":"2.0","id":4,"method":"ping"}"#);
    input.extend_from_slice(b"\n");

    let responses = serve_bytes(home.path(), &[], &input);
    assert_eq!(responses.len(), 4, "{responses:#?}");

    assert_eq!(responses[0]["error"]["code"], -32600, "{}", responses[0]);
    assert_eq!(responses[0]["id"], Value::Null);
    assert_eq!(responses[1]["error"]["code"], -32700, "{}", responses[1]);
    assert_eq!(responses[1]["id"], Value::Null);
    assert_eq!(responses[2]["error"]["code"], -32600, "{}", responses[2]);
    assert_eq!(responses[2]["id"], 3);
    assert_eq!(
        responses[3],
        json!({ "jsonrpc": "2.0", "id": 4, "result": {} })
    );
}

/// What a tool answers matches the `outputSchema` it was listed with:
/// every required key there, no key it does not declare.
#[test]
fn a_call_answers_the_payload_its_output_schema_describes() {
    let home = account();
    let responses = serve(
        home.path(),
        &[],
        &[
            list(1),
            call(2, "calendar_list", json!({})),
            call(3, "event_list", json!({ "calendar": "personal" })),
        ],
    );

    for (response, name, key) in [
        (&responses[1], "calendar_list", "calendars"),
        (&responses[2], "event_list", "events"),
    ] {
        let result = &response["result"];
        assert_ne!(result["isError"], true, "{response}");

        let content = result["structuredContent"].as_object().unwrap();
        let schema = &tool(&responses[0], name)["outputSchema"];
        let declared = schema["properties"].as_object().unwrap();

        for required in schema["required"].as_array().unwrap() {
            let required = required.as_str().unwrap();
            assert!(content.contains_key(required), "{name}: {required}");
        }
        for key in content.keys() {
            assert!(declared.contains_key(key), "{name}: {key}");
        }

        assert!(!content[key].as_array().unwrap().is_empty(), "{response}");
    }

    let events = responses[2]["result"]["structuredContent"]["events"]
        .as_array()
        .unwrap();
    let mut ids: Vec<&str> = events
        .iter()
        .map(|event| event["id"].as_str().unwrap())
        .collect();
    ids.sort_unstable();
    assert_eq!(ids, ["review", "standup"]);
    assert!(responses[2]["result"]["structuredContent"]["nextPage"].is_null());
}

#[test]
fn an_unknown_argument_is_a_tool_error() {
    let home = account();
    let responses = serve(
        home.path(),
        &[],
        &[
            call(1, "event_list", json!({ "query": "standup" })),
            call(2, "event_create", json!({ "summary": "Nope" })),
        ],
    );

    let error = refusal(&responses[0]);
    assert!(error.contains("Unknown argument `query`"), "{error}");
    assert!(error.contains("`calendar`"), "{error}");
    assert_eq!(report(&responses[0])["code"], "usage-invalid");

    assert_eq!(responses[1]["error"]["code"], -32602, "{}", responses[1]);
}

/// A value the schema lets through but the command refuses is a tool
/// error coded `usage-invalid`.
#[test]
fn a_value_the_command_refuses_is_a_usage_invalid_tool_error() {
    let home = account();
    let responses = serve(
        home.path(),
        &[],
        &[call(1, "event_list", json!({ "from": "someday" }))],
    );

    let error = refusal(&responses[0]);
    assert!(error.contains("'someday'"), "{error}");
    assert!(!error.starts_with("error:"), "{error}");
    assert_eq!(report(&responses[0])["code"], "usage-invalid");
}

#[test]
fn an_ical_source_is_taken_as_text_never_as_a_local_path() {
    let home = account();
    let secret = home.path().join("secret.ics");
    fs::write(&secret, event("secret", "")).unwrap();

    let responses = serve(
        home.path(),
        &["--allow-write"],
        &[
            call(1, "item_create", json!({ "ical": secret })),
            call(2, "item_create", json!({ "ical": "-" })),
            call(3, "item_create", json!({ "ical": event("lunch", "") })),
        ],
    );

    let error = refusal(&responses[0]);
    assert!(error.contains("neither a readable file"), "{error}");

    let report = &responses[1]["result"]["content"][0]["text"];
    let report: Value = serde_json::from_str(report.as_str().unwrap()).unwrap();
    assert_eq!(report["code"], "stdin-taken", "{report}");

    assert_ne!(responses[2]["result"]["isError"], true, "{}", responses[2]);

    let stored = stored(home.path());
    assert_eq!(stored.len(), 3, "{stored:?}");
    let contents: Vec<String> = stored
        .iter()
        .map(|name| fs::read_to_string(home.path().join("calendars/personal").join(name)).unwrap())
        .collect();
    assert!(!contents.iter().any(|ics| ics.contains("secret")));
}

/// Without `--allow-send` no write reaches an item naming attendees, the
/// one handed over or the one stored; with it, the same calls land.
#[test]
fn a_write_naming_attendees_needs_allow_send() {
    let home = account();
    let invitation = event("invite", GUEST);
    let calls = [
        call(1, "event_create", json!({ "ical": invitation })),
        call(2, "item_create", json!({ "ical": invitation })),
        call(
            3,
            "event_update",
            json!({ "event-id": "review", "summary": "Moved" }),
        ),
        call(
            4,
            "item_update",
            json!({ "item-id": "review", "ical": event("review", "") }),
        ),
        call(5, "event_delete", json!({ "event-id": "review" })),
        call(6, "item_delete", json!({ "item-id": "review" })),
        call(
            7,
            "event_create",
            json!({ "summary": "Lunch", "start": "2026-11-13T12:00" }),
        ),
        call(8, "event_delete", json!({ "event-id": "standup" })),
    ];

    let before = fs::read_to_string(home.path().join("calendars/personal/review.ics")).unwrap();
    let responses = serve(home.path(), &["--allow-write"], &calls);
    assert_eq!(responses.len(), 8, "{responses:#?}");

    for response in &responses[..6] {
        let error = refusal(response);
        assert!(error.contains("--allow-send"), "{error}");
        assert!(error.contains("ATTENDEE"), "{error}");
    }

    assert_ne!(responses[6]["result"]["isError"], true, "{}", responses[6]);
    assert_ne!(responses[7]["result"]["isError"], true, "{}", responses[7]);

    let after = fs::read_to_string(home.path().join("calendars/personal/review.ics")).unwrap();
    assert_eq!(after, before);
    let kept = stored(home.path());
    assert_eq!(kept.len(), 2, "{kept:?}");
    assert!(kept.contains(&String::from("review.ics")), "{kept:?}");
    assert!(!kept.contains(&String::from("standup.ics")), "{kept:?}");

    let flags = ["--allow-write", "--allow-send"];
    let responses = serve(home.path(), &flags, &[calls[0].clone(), calls[4].clone()]);
    assert_ne!(responses[0]["result"]["isError"], true, "{}", responses[0]);
    assert_ne!(responses[1]["result"]["isError"], true, "{}", responses[1]);

    let stored = stored(home.path());
    assert_eq!(stored.len(), 2, "{stored:?}");
    assert!(!stored.contains(&String::from("review.ics")), "{stored:?}");
}

/// A pimdir store takes no calendar write, so a server over it lists none.
#[cfg(feature = "pimdir")]
#[test]
fn a_tool_the_account_cannot_serve_is_not_listed() {
    let home = TempDir::new().unwrap();
    let root = home.path().join("store");
    let toml = format!(
        "[accounts.perso]\ndefault = true\npimdir.root = {:?}\n",
        root.display().to_string(),
    );
    fs::write(home.path().join("config.toml"), toml).unwrap();

    let responses = serve(home.path(), &["--allow-write"], &[list(1)]);
    let names = names(&responses[0]);

    assert!(names.contains(&"calendar_list"), "{names:?}");
    assert!(names.contains(&"event_create"), "{names:?}");
    for name in ["calendar_create", "calendar_update", "calendar_delete"] {
        assert!(!names.contains(&name), "{name}: {names:?}");
    }
}
