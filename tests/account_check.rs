//! # Account check
//!
//! The built binary checking an account whose vdir home exists and whose
//! pimdir store does not: the report printed whole, in text and in JSON,
//! and the exit status 3, or 0 once the failing backend is left out.

#![cfg(all(feature = "vdir", feature = "pimdir"))]

use std::{
    fs,
    path::Path,
    process::{Command, Output, Stdio},
};

use serde_json::{Value, json};
use tempfile::TempDir;

/// The account `test`, its vdir home in `home` and its pimdir root a
/// directory `home` lacks, and its configuration file.
fn account() -> TempDir {
    let home = TempDir::new().unwrap();
    let config = format!(
        "[accounts.test]\ndefault = true\nvdir.home-dir = {:?}\npimdir.root = {:?}\n",
        home.path().display().to_string(),
        home.path().join("missing").display().to_string(),
    );
    fs::write(home.path().join("config.toml"), config).unwrap();

    home
}

/// Runs the binary with `args` over the account in `home`.
fn run(home: &Path, args: &[&str]) -> Output {
    let config = home.join("config.toml");

    Command::new(env!("CARGO_BIN_EXE_calendula"))
        .args(["-c", config.to_str().unwrap()])
        .args(args)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

#[test]
fn a_failed_backend_prints_the_report_and_exits_3() {
    let home = account();
    let output = run(home.path(), &["account", "check"]);
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(output.status.code(), Some(3), "{stdout}");

    let mut lines = stdout.lines();
    assert_eq!(lines.next(), Some("Account:  test"), "{stdout}");
    assert_eq!(lines.next(), Some("vdir:     OK"), "{stdout}");
    let pimdir = lines.next().unwrap_or_default();
    assert!(pimdir.starts_with("pimdir:   FAIL ("), "{stdout}");
    assert_eq!(lines.next(), None, "{stdout}");
}

/// The report carries the stable code of a failure a script acts on.
#[test]
fn a_failed_backend_is_the_json_result_with_its_code() {
    let home = account();
    let output = run(home.path(), &["--json", "account", "check"]);
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(output.status.code(), Some(3), "{json}");
    assert_eq!(json["account"], "test", "{json}");

    let vdir = json!({"backend": "vdir", "ok": true, "code": null, "error": null});
    assert_eq!(json["backends"][0], vdir, "{json}");

    let pimdir = &json["backends"][1];
    assert_eq!(pimdir["backend"], "pimdir", "{json}");
    assert_eq!(pimdir["ok"], false, "{json}");
    assert_eq!(pimdir["code"], "store-uncreated", "{json}");
    assert!(pimdir["error"].is_string(), "{json}");
}

#[test]
fn a_passing_check_exits_0() {
    let home = account();
    let output = run(home.path(), &["--backend", "vdir", "account", "check"]);
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(output.status.code(), Some(0), "{stdout}");
    assert_eq!(stdout, "Account:  test\nvdir:     OK\n");
}
