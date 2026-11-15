//! # Account resolution
//!
//! The built binary taking the account a command runs against: the three
//! ways pimalaya-config refuses one, each coded under `--json`, `default`
//! read as a name like any other, and the block of a backend this build
//! leaves out, loaded so one file serves every build, then refused with
//! `backend-not-built` by a command reaching it.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use serde_json::Value;
use tempfile::TempDir;

/// Writes `toml` as the configuration of a fresh directory.
fn config(toml: &str) -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.toml");
    fs::write(&path, toml).unwrap();
    (dir, path)
}

/// Runs the binary with `args` over `config`, its directory as home.
fn run(config: &Path, args: &[&str]) -> Output {
    let home = config.parent().unwrap();

    Command::new(env!("CARGO_BIN_EXE_calendula"))
        .arg("-c")
        .arg(config)
        .args(args)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .env_remove("XDG_CONFIG_DIRS")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

/// The JSON a run printed, failing unless it exited with `status`.
fn json(out: &Output, status: i32) -> Value {
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(status), "{stdout} {stderr}");
    serde_json::from_slice(&out.stdout).unwrap_or_else(|err| panic!("{err}: {stdout}"))
}

/// The failure object of a `--json` run, failing unless it exited 1 with
/// `code`, its message holding `text`.
fn refused(out: &Output, code: &str, text: &str) -> Value {
    let error = json(out, 1);
    assert_eq!(error["code"], code, "{error}");
    let message = error["error"].as_str().unwrap_or_default();
    assert!(message.contains(text), "{error}");
    error
}

#[test]
fn an_unknown_account_is_coded_account_missing() {
    let (_dir, path) = config("[accounts.work]\ndefault = true\n");
    let out = run(&path, &["--json", "-a", "nope", "account", "capability"]);
    refused(&out, "account-missing", "holds: work");
}

/// `-a default` once took the account marked default; it now names an
/// account, the message saying how to take the marked one.
#[test]
fn default_is_a_name_like_any_other() {
    let (_dir, path) = config("[accounts.work]\ndefault = true\n");
    let out = run(&path, &["--json", "-a", "default", "account", "capability"]);
    refused(
        &out,
        "account-missing",
        "omit `-a` to take the account marked default",
    );

    let toml = "[accounts.default]\n[accounts.work]\ndefault = true\n";
    let (_dir, path) = config(toml);
    let out = run(&path, &["--json", "-a", "default", "account", "capability"]);
    assert_eq!(json(&out, 0)["account"], "default");
}

#[test]
fn no_default_account_is_coded_account_default_missing() {
    let (_dir, path) = config("[accounts.work]\n[accounts.perso]\n");
    let out = run(&path, &["--json", "account", "capability"]);
    refused(&out, "account-default-missing", "No default account");
}

#[test]
fn several_default_accounts_are_coded_account_default_several() {
    let toml = "[accounts.work]\ndefault = true\n[accounts.perso]\ndefault = true\n";
    let (_dir, path) = config(toml);
    let out = run(&path, &["--json", "account", "capability"]);
    refused(&out, "account-default-several", "perso, work");
}

/// The account `test`, marked default, carrying a gcal block this build
/// leaves out, then `lines`.
#[cfg(not(feature = "gcal"))]
fn gcal_account(lines: &str) -> (TempDir, PathBuf) {
    let toml = "[accounts.test]\ndefault = true\ngcal.calendar-id = \"primary\"\n";
    config(&format!("{toml}{lines}"))
}

/// A block of a backend this build leaves out loads, and a unified
/// command reaching it fails naming the feature bringing it back.
#[cfg(not(feature = "gcal"))]
#[test]
fn a_backend_left_out_fails_coded_backend_not_built() {
    let (_dir, path) = gcal_account("");
    let out = run(&path, &["--json", "calendar", "list"]);
    let error = refused(&out, "backend-not-built", "rebuild with `--features gcal`");
    let message = error["error"].as_str().unwrap();
    assert!(!message.contains("--backend"), "{message}");

    let out = run(&path, &["--json", "account", "check"]);
    let report = json(&out, 3);
    let check = &report["backends"][0];
    assert_eq!(check["backend"], "gcal", "{report}");
    assert_eq!(check["code"], "backend-not-built", "{report}");

    let out = run(&path, &["--backend", "gcal", "account", "check"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "{stderr}");
    assert!(
        stderr.contains("rebuild with `--features gcal`"),
        "{stderr}"
    );
}

/// The block still refuses a key no build knows, beside it.
#[cfg(not(feature = "gcal"))]
#[test]
fn a_typo_beside_a_backend_left_out_is_refused() {
    let (_dir, path) = gcal_account("gcall.calendar-id = \"primary\"\n");
    let out = run(&path, &["--json", "account", "list"]);
    let error = json(&out, 1);
    let cause = error["sources"][0].as_str().unwrap_or_default();
    assert!(cause.contains("unknown field `gcall`"), "{error}");
}

/// `account list` names the backend left out rather than leaves it unsaid,
/// and `account capability`, which reaches it, fails as a unified command
/// does.
#[cfg(not(feature = "gcal"))]
#[test]
fn the_account_commands_meet_a_backend_left_out() {
    let (_dir, path) = gcal_account("");

    let list = json(&run(&path, &["--json", "account", "list"]), 0);
    let account = &list["accounts"][0];
    assert_eq!(account["backends"], serde_json::json!([]), "{list}");
    assert_eq!(account["notBuilt"], serde_json::json!(["gcal"]), "{list}");

    let out = run(&path, &["account", "list"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("gcal (not built)"), "{text}");

    let out = run(&path, &["--json", "account", "capability"]);
    refused(&out, "backend-not-built", "rebuild with `--features gcal`");
}

/// `auto` stops on the backend left out rather than falls through to the
/// next one, naming the one `--backend` reaches instead.
#[cfg(all(feature = "msgraph", not(feature = "gcal")))]
#[test]
fn auto_stops_on_a_backend_left_out_naming_the_built_one() {
    let (_dir, path) = gcal_account("msgraph.auth.token.raw = \"made-up\"\n");
    let out = run(&path, &["--json", "calendar", "list"]);
    refused(&out, "backend-not-built", "(`--backend msgraph`)");

    let out = run(&path, &["--json", "account", "capability"]);
    refused(&out, "backend-not-built", "(`--backend msgraph`)");
}
