//! # Exit contract
//!
//! The built binary's exit statuses beyond success and failure: a command
//! line the grammar refuses exits 2, its failure object coded
//! `usage-invalid` under `--json`, and clap refuses it before any
//! configuration is read. A command that printed its result without fully
//! succeeding exits 3, which tests/account_check.rs and tests/event_list.rs
//! hold.

use std::{
    path::Path,
    process::{Command, Output, Stdio},
};

use serde_json::{Value, json};
use tempfile::TempDir;

/// Runs the binary with `args` in `home`, its stdin closed.
fn run(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_calendula"))
        .args(args)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .env_remove("XDG_CONFIG_DIRS")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn a_refused_command_line_under_json_prints_the_usage_failure_object() {
    let home = TempDir::new().unwrap();
    let output = run(home.path(), &["--json", "--nope", "calendar", "list"]);
    let stdout = text(&output.stdout);

    assert_eq!(output.status.code(), Some(2), "{stdout}");
    assert_eq!(stdout.lines().count(), 1, "{stdout}");

    let json: Value = serde_json::from_str(&stdout).unwrap();
    let error = json["error"].as_str().unwrap();
    assert_eq!(json["code"], "usage-invalid", "{json}");
    assert!(error.starts_with("unexpected argument '--nope'"), "{error}");
    assert!(!error.contains('\x1b'), "{error}");
    assert_eq!(json["sources"], json!([]), "{json}");
    assert_eq!(json["backtrace"], Value::Null, "{json}");
    assert!(output.stderr.is_empty(), "{}", text(&output.stderr));
}

#[test]
fn a_refused_command_line_without_json_is_clap_rendering_on_stderr() {
    let home = TempDir::new().unwrap();
    let output = run(home.path(), &["--nope", "calendar", "list"]);
    let stderr = text(&output.stderr);

    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(output.stdout.is_empty(), "{}", text(&output.stdout));
    assert!(stderr.starts_with("error: unexpected argument"), "{stderr}");
}

/// A `--json` after `--` is a positional, not the flag.
#[test]
fn a_json_word_after_the_separator_leaves_the_refusal_on_stderr() {
    let home = TempDir::new().unwrap();
    let output = run(home.path(), &["event", "list", "--nope", "--", "--json"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty(), "{}", text(&output.stdout));
    assert!(!output.stderr.is_empty());
}

/// `completion`, `manual` and `json-schema` with no argument fail as any
/// command does, the error on stdout, rather than printing their help.
#[test]
fn a_standard_command_without_its_argument_fails_on_stdout() {
    let home = TempDir::new().unwrap();

    for command in ["completion", "manual", "json-schema"] {
        let output = run(home.path(), &[command]);
        let stdout = text(&output.stdout);

        assert_eq!(output.status.code(), Some(1), "{command}: {stdout}");
        assert!(stdout.contains("--dir"), "{command}: {stdout}");
        assert!(!stdout.contains("Usage:"), "{command}: {stdout}");
        assert!(output.stderr.is_empty(), "{}", text(&output.stderr));
    }
}
