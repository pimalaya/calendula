//! # Usage
//!
//! The built binary refusing a command line: exit 2, the failure object
//! coded `usage-invalid` on stdout under `--json`, clap's rendering on
//! stderr otherwise. clap refuses before any configuration is read.

use std::process::{Command, Output, Stdio};

use serde_json::{Value, json};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_calendula"))
        .args(args)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

#[test]
fn a_refused_command_line_under_json_prints_usage_invalid_and_exits_2() {
    let output = run(&["--json", "--nope", "calendar", "list"]);
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(output.status.code(), Some(2), "{stdout}");
    assert_eq!(stdout.lines().count(), 1, "{stdout}");

    let json: Value = serde_json::from_str(&stdout).unwrap();
    let error = json["error"].as_str().unwrap();
    assert_eq!(json["code"], "usage-invalid", "{json}");
    assert!(error.starts_with("unexpected argument '--nope'"), "{error}");
    assert!(!error.contains('\x1b'), "{error}");
    assert_eq!(json["sources"], json!([]), "{json}");
    assert_eq!(json["backtrace"], Value::Null, "{json}");
}

#[test]
fn a_refused_command_line_prints_clap_on_stderr_and_exits_2() {
    let output = run(&["--nope", "calendar", "list"]);
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(output.stdout.is_empty(), "{stderr}");
    assert!(stderr.starts_with("error: unexpected argument"), "{stderr}");
}
