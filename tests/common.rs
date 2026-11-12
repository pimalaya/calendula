//! # Shared test helpers
//!
//! The account configuration the live suites write, the built binary run
//! against it with an access token in its environment, the provider
//! sections of config.sample.toml, and the environment the credentialed
//! suites read their secrets from.

#![allow(dead_code)]

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use serde_json::Value;

/// The variable the binary reads its access token from.
pub const TOKEN_VAR: &str = "CALENDULA_TEST_TOKEN";

/// The token command of every account the suites write: the token
/// reaches the binary through its environment, never its arguments nor
/// its configuration.
pub const TOKEN_COMMAND: &str = r#"["printenv", "CALENDULA_TEST_TOKEN"]"#;

/// A token no provider ever issued.
pub const MADE_UP: &str = "made-up";

/// What calendula says of a request a provider refuses, whichever the
/// backend.
pub const REFUSED: &str = "HTTP 401";

/// Writes a configuration in `dir` holding `name` as the default
/// account, made of `lines`, and returns its path.
pub fn write_config(dir: &Path, name: &str, lines: &str) -> PathBuf {
    let config = dir.join("config.toml");
    let account = format!("[accounts.{name}]\ndefault = true\n{lines}\n");
    fs::write(&config, account).unwrap();
    config
}

/// Runs the built binary against `config`, `token` in its environment,
/// the configuration's directory as its home and its stdin closed.
pub fn calendula(config: &Path, token: &str, args: &[&str]) -> Output {
    let home = config.parent().unwrap();

    Command::new(env!("CARGO_BIN_EXE_calendula"))
        .arg("-c")
        .arg(config)
        .args(args)
        .env(TOKEN_VAR, token)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .env_remove("XDG_CONFIG_DIRS")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

/// What a run shows when an assertion fails: its exit code, its stdout,
/// where failures are printed, and its stderr. The token is in neither,
/// the binary reading it from its environment only.
pub fn failure(out: &Output) -> String {
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let code = out.status.code();
    format!("exit {code:?}, stdout: {stdout}, stderr: {stderr}")
}

/// The JSON a `--json` run printed, failing unless it exited with
/// `code`.
pub fn json(out: &Output, code: i32) -> Value {
    assert_eq!(out.status.code(), Some(code), "{}", failure(out));
    serde_json::from_slice(&out.stdout).unwrap_or_else(|err| panic!("{err}: {}", failure(out)))
}

/// The failure object a `--json` run printed, failing unless it exited
/// with 1 and its message holds `refusal`.
pub fn json_failure(out: &Output, refusal: &str) -> Value {
    let error = json(out, 1);
    let message = error["error"].as_str().unwrap_or_default();
    assert!(message.contains(refusal), "{error}");
    assert!(error["sources"].is_array(), "{error}");
    error
}

/// The report a `--json` `account check` run printed, failing unless it
/// exited with 2, the partial outcome, and names `backend` alone, failed
/// for `refusal`.
pub fn json_check_failure(out: &Output, backend: &str, refusal: &str) -> Value {
    let report = json(out, 2);
    let backends = report["backends"].as_array().unwrap();
    assert_eq!(backends.len(), 1, "{report}");

    let check = &backends[0];
    let message = check["error"].as_str().unwrap_or_default();
    assert_eq!(check["backend"], backend, "{report}");
    assert_eq!(check["ok"], false, "{report}");
    assert!(message.contains(refusal), "{report}");
    report
}

/// Fails unless `report` is the `account check` output of one backend,
/// `backend`, that answered.
pub fn assert_check(report: &Value, backend: &str) {
    let backends = report["backends"].as_array().unwrap();
    assert_eq!(backends.len(), 1, "{report}");
    assert_eq!(backends[0]["backend"], backend, "{report}");
    assert_eq!(backends[0]["ok"], true, "{report}");
    assert!(backends[0]["code"].is_null(), "{report}");
    assert!(backends[0]["error"].is_null(), "{report}");
}

/// Fails unless `list` is a `calendar list` output whose calendars are
/// each of the shape the binary prints, and returns their ids. A failure
/// names the faulty member, never its value: a calendar is personal data.
pub fn assert_calendars(list: &Value) -> Vec<String> {
    let calendars = list["calendars"].as_array().unwrap();
    assert!(!calendars.is_empty(), "no calendar");

    for (i, calendar) in calendars.iter().enumerate() {
        assert!(calendar["id"].is_string(), "calendar {i}: no string id");
        assert!(calendar["name"].is_string(), "calendar {i}: no string name");
        assert!(calendar["default"].is_boolean(), "calendar {i}: no default");

        for key in ["description", "color"] {
            let value = &calendar[key];
            let valid = value.is_string() || value.is_null();
            assert!(valid, "calendar {i}: {key} neither a string nor null");
        }
    }

    let ids = calendars.iter().filter_map(|c| c["id"].as_str());
    ids.map(str::to_owned).collect()
}

/// Fails unless `list` is an `event list` output of at most `size`
/// events, each of the shape the binary prints. A failure names the
/// faulty member, never its value: an event is personal data.
pub fn assert_events(list: &Value, size: usize) {
    let events = list["events"].as_array().unwrap();
    let count = events.len();
    let next = &list["nextPage"];

    assert!(count <= size, "{count} events");
    assert!(next.is_string() || next.is_null(), "nextPage: {next}");

    for (i, event) in events.iter().enumerate() {
        let strings = [
            "id",
            "uid",
            "summary",
            "description",
            "location",
            "start",
            "end",
        ];

        for key in strings {
            assert!(event[key].is_string(), "event {i}: {key} not a string");
        }

        for key in ["allDay", "zoneAssumed", "recurring"] {
            assert!(event[key].is_boolean(), "event {i}: {key} not a bool");
        }

        assert!(event["sequence"].is_u64(), "event {i}: sequence");

        let optional = [
            "etag",
            "recurrenceId",
            "startsAt",
            "endsAt",
            "timeZone",
            "status",
            "transparency",
            "busyStatus",
            "onlineMeetingUrl",
        ];

        for key in optional {
            let value = &event[key];
            let valid = value.is_string() || value.is_null();
            assert!(valid, "event {i}: {key} neither a string nor null");
        }

        let organizer = &event["organizer"];
        let valid = organizer.is_null() || organizer["email"].is_string();
        assert!(valid, "event {i}: organizer without an email");

        let attendees = event["attendees"].as_array();
        let attendees = attendees.unwrap_or_else(|| panic!("event {i}: attendees"));

        for (j, attendee) in attendees.iter().enumerate() {
            let email = attendee["email"].is_string();
            assert!(email, "event {i}: attendee {j} without an email");
        }

        assert!(event["alarms"].is_array(), "event {i}: alarms");
    }
}

/// The commented section `name` of config.sample.toml, uncommented,
/// each line setting a key of `overrides` set to its value instead.
pub fn sample_section(name: &str, overrides: &[(&str, &str)]) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.sample.toml");
    let sample = fs::read_to_string(path).unwrap();
    let heading = format!("# {name}");

    let mut lines = sample.lines().skip_while(|line| *line != heading).skip(2);
    let mut section = String::new();

    for line in lines.by_ref().take_while(|line| !line.is_empty()) {
        let line = line.strip_prefix('#').unwrap_or(line);
        let key = line.split(" = ").next().unwrap_or_default();

        match overrides.iter().find(|(k, _)| *k == key) {
            Some((key, value)) => section.push_str(&format!("{key} = {value}\n")),
            None => section.push_str(&format!("{line}\n")),
        }
    }

    assert!(
        !section.is_empty(),
        "no {name} section in config.sample.toml"
    );
    section
}

/// The environment variable `name`, an empty value counting as unset,
/// since a workflow maps a missing secret to an empty one.
pub fn env_var(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.is_empty())
}

/// [`env_var`] of each of `names`, failing the test with every one of
/// them unset.
pub fn require_env<const N: usize>(names: [&str; N]) -> [String; N] {
    let values = names.map(env_var);
    let unset: Vec<_> = names
        .iter()
        .zip(&values)
        .filter(|(_, value)| value.is_none())
        .map(|(name, _)| *name)
        .collect();

    assert!(
        unset.is_empty(),
        "set {} to run this test",
        unset.join(", ")
    );
    values.map(Option::unwrap)
}

/// The content of the file the `{name}_FILE` variable names, or else of
/// the variable `name`.
pub fn env_content(name: &str) -> Option<String> {
    match env_var(&format!("{name}_FILE")) {
        Some(path) => Some(fs::read_to_string(path).unwrap()),
        None => env_var(name),
    }
}
