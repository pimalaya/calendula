//! # Capability
//!
//! The built binary refusing what the backend of its account does not
//! serve with `capability-missing`, before it reads, opens or connects
//! anything; `account capability` listing what each backend of the
//! account serves; and the `Backends:` footer of a unified command's
//! help.
//!
//! A backend left unreached shows as a credential command that never ran
//! or a store root never created.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

#[cfg(any(
    feature = "vdir",
    feature = "caldav",
    feature = "pimdir",
    feature = "gcal",
    feature = "msgraph"
))]
use serde_json::Value;
use tempfile::TempDir;

/// Writes the default account `test`, made of `lines`, and returns the
/// path of its configuration.
fn write_config(dir: &Path, lines: &str) -> PathBuf {
    let config = dir.join("config.toml");
    fs::write(&config, format!("[accounts.test]\ndefault = true\n{lines}")).unwrap();
    config
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
        .env("TZ", "UTC")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

/// A credential command printing `secret` after writing a line to
/// `marker`, so a run that resolved the credential leaves the file.
#[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
fn marker_command(marker: &Path) -> String {
    format!("echo run >> '{}'; echo secret", marker.display())
}

/// The message of a `--json` run refused with `capability-missing`,
/// naming `capability`.
#[cfg(any(
    feature = "vdir",
    feature = "caldav",
    feature = "pimdir",
    feature = "gcal",
    feature = "msgraph"
))]
fn refused(out: &Output, capability: &str) -> String {
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "{stdout}");

    let error: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(error["code"], "capability-missing", "{error}");

    let message = error["error"].as_str().unwrap().to_owned();
    assert!(message.contains(&format!("`{capability}`")), "{message}");
    message
}

#[cfg(feature = "vdir")]
const EVENT: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\n\
                     BEGIN:VEVENT\r\nUID:1@example.org\r\nDTSTAMP:20261101T000000Z\r\n\
                     DTSTART:20261112T090000Z\r\nSUMMARY:Standup\r\nEND:VEVENT\r\n\
                     END:VCALENDAR\r\n";

/// A vdir home holding the calendar `cal` and its event `1`, and the
/// configuration line naming it.
#[cfg(feature = "vdir")]
fn vdir_home(dir: &Path) -> String {
    let cal = dir.join("cal");
    fs::create_dir(&cal).unwrap();
    fs::write(cal.join("1.ics"), EVENT).unwrap();
    format!("vdir.home-dir = {:?}\n", dir.display().to_string())
}

/// A vdir item has no version: a write gated on one is refused before
/// the item is read, and the item keeps its bytes.
#[cfg(feature = "vdir")]
#[test]
fn a_vdir_write_gated_on_a_version_is_refused() {
    let dir = TempDir::new().unwrap();
    let config = write_config(dir.path(), &vdir_home(dir.path()));

    let args = [
        "--json",
        "item",
        "delete",
        "-k",
        "cal",
        "--if-match",
        "x",
        "1",
    ];
    let message = refused(&run(&config, &args), "item.remove.if-match");
    assert!(message.contains("vdir backend"), "{message}");
    assert!(message.contains("--if-match"), "{message}");

    let args = [
        "--json",
        "event",
        "update",
        "-k",
        "cal",
        "--if-match",
        "x",
        "--summary",
        "Retro",
        "1",
    ];
    refused(&run(&config, &args), "item.update.if-match");

    let stored = fs::read_to_string(dir.path().join("cal").join("1.ics")).unwrap();
    assert_eq!(stored, EVENT);
}

/// The refusal names another backend of the account serving what the
/// first one does not, which `--backend` picks, and reaches neither.
#[cfg(all(feature = "vdir", feature = "caldav"))]
#[test]
fn a_refusal_names_the_backend_serving_it() {
    let dir = TempDir::new().unwrap();
    let marker = dir.path().join("marker");
    let lines = format!(
        "{}caldav.home = \"http://127.0.0.1:1/\"\n\
         caldav.auth.basic.username = \"test\"\n\
         caldav.auth.basic.password.command = {:?}\n",
        vdir_home(dir.path()),
        marker_command(&marker),
    );
    let config = write_config(dir.path(), &lines);

    let args = [
        "--json",
        "item",
        "delete",
        "-k",
        "cal",
        "--if-match",
        "x",
        "1",
    ];
    let message = refused(&run(&config, &args), "item.remove.if-match");
    assert!(message.contains("vdir backend"), "{message}");
    assert!(message.contains("`--backend caldav`"), "{message}");

    assert!(!marker.exists());
    assert!(dir.path().join("cal").join("1.ics").exists());
}

/// CalDAV on a loopback port nothing listens on: a flag it does not
/// serve is refused before its password is even resolved.
#[cfg(feature = "caldav")]
#[test]
fn a_caldav_flag_it_does_not_serve_is_refused_before_connecting() {
    let dir = TempDir::new().unwrap();
    let marker = dir.path().join("marker");
    let lines = format!(
        "caldav.home = \"http://127.0.0.1:1/\"\n\
         caldav.auth.basic.username = \"test\"\n\
         caldav.auth.basic.password.command = {:?}\n",
        marker_command(&marker),
    );
    let config = write_config(dir.path(), &lines);

    let args = [
        "--json",
        "event",
        "create",
        "-k",
        "cal",
        "--summary",
        "Sync",
        "--online-meeting",
    ];
    let message = refused(&run(&config, &args), "online-meeting");
    assert!(message.contains("caldav backend"), "{message}");

    let args = [
        "--json",
        "event",
        "delete",
        "-k",
        "cal",
        "--send-updates",
        "none",
        "1",
    ];
    refused(&run(&config, &args), "scheduling.send-updates");

    assert!(!marker.exists());
}

/// A pimdir store's calendars come from its sync engine: a calendar write
/// is refused before the store is opened.
#[cfg(feature = "pimdir")]
#[test]
fn a_pimdir_calendar_write_is_refused_before_opening_the_store() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("store");
    let lines = format!("pimdir.root = {:?}\n", root.display().to_string());
    let config = write_config(dir.path(), &lines);

    let args = ["--json", "calendar", "create", "work", "Work"];
    refused(&run(&config, &args), "calendar.create");

    let args = ["--json", "calendar", "delete", "-k", "work"];
    refused(&run(&config, &args), "calendar.delete");

    assert!(!root.exists());
}

/// gcal creates no online meeting: the flag is refused before the token
/// command runs.
#[cfg(feature = "gcal")]
#[test]
fn a_gcal_online_meeting_is_refused_before_the_token_is_resolved() {
    let dir = TempDir::new().unwrap();
    let marker = dir.path().join("marker");
    let lines = format!("gcal.auth.token.command = {:?}\n", marker_command(&marker));
    let config = write_config(dir.path(), &lines);

    let args = [
        "--json",
        "event",
        "create",
        "-k",
        "primary",
        "--summary",
        "Sync",
        "--online-meeting",
    ];
    let message = refused(&run(&config, &args), "online-meeting");
    assert!(message.contains("gcal backend"), "{message}");

    assert!(!marker.exists());
}

/// A Graph calendar carries no description: one given is refused before
/// the token command runs.
#[cfg(feature = "msgraph")]
#[test]
fn a_graph_calendar_description_is_refused_before_the_token_is_resolved() {
    let dir = TempDir::new().unwrap();
    let marker = dir.path().join("marker");
    let lines = format!(
        "msgraph.auth.token.command = {:?}\n",
        marker_command(&marker)
    );
    let config = write_config(dir.path(), &lines);

    let args = ["--json", "calendar", "create", "work", "Work", "-d", "Mine"];
    let message = refused(&run(&config, &args), "calendar.description");
    assert!(message.contains("msgraph backend"), "{message}");

    assert!(!marker.exists());
}

#[cfg(feature = "vdir")]
#[test]
fn account_capability_lists_what_the_backend_serves() {
    let dir = TempDir::new().unwrap();
    let config = write_config(dir.path(), &vdir_home(dir.path()));

    let out = run(&config, &["account", "capability"]);
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success(), "{stdout}");

    let expected = "\
CAPABILITY               BACKEND  LEVEL  DETAIL
calendar.list            vdir     full   -
calendar.create          vdir     full   -
calendar.rename          vdir     full   -
calendar.delete          vdir     full   -
calendar.description     vdir     full   -
calendar.color           vdir     full   -
item.list                vdir     full   -
item.read                vdir     full   -
item.add                 vdir     full   -
item.update              vdir     full   -
item.remove              vdir     full   -
item.update.if-match     vdir     none   the backend exposes no version to compare, so `--if-match` is refused
item.remove.if-match     vdir     none   the backend exposes no version to compare, so `--if-match` is refused
online-meeting           vdir     none   `--online-meeting` needs the pimdir backend, whose sync engine creates the meeting
scheduling.send-updates  vdir     none   a vdir store notifies nobody; `--send-updates` needs the gcal backend
";
    assert_eq!(stdout, expected);
}

/// Every backend the account configures is listed, in the order `auto`
/// tries them, and `--backend` keeps one; the pimdir store is not opened.
#[cfg(all(feature = "vdir", feature = "pimdir"))]
#[test]
fn account_capability_lists_each_backend_of_the_account() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("store");
    let lines = format!(
        "{}pimdir.root = {:?}\n",
        vdir_home(dir.path()),
        root.display().to_string()
    );
    let config = write_config(dir.path(), &lines);

    let out = run(&config, &["--json", "account", "capability"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();

    assert_eq!(json["account"], "test", "{json}");
    let backends = json["backends"].as_array().unwrap();
    assert_eq!(backends.len(), 2, "{json}");
    assert_eq!(backends[0]["backend"], "vdir", "{json}");
    assert_eq!(backends[1]["backend"], "pimdir", "{json}");

    let create = &backends[1]["capabilities"][1];
    assert_eq!(create["name"], "calendar.create", "{json}");
    assert_eq!(create["level"], "none", "{json}");
    assert!(create["detail"].is_string(), "{json}");

    let list = &backends[1]["capabilities"][0];
    let full = serde_json::json!({"name": "calendar.list", "level": "full", "detail": null});
    assert_eq!(*list, full, "{json}");

    let out = run(
        &config,
        &["--json", "--backend", "pimdir", "account", "capability"],
    );
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    let backends = json["backends"].as_array().unwrap();
    assert_eq!(backends.len(), 1, "{json}");
    assert_eq!(backends[0]["backend"], "pimdir", "{json}");

    assert!(!root.exists());
}

/// The footer is generated from the tables, the same in every build.
#[test]
fn the_help_names_what_each_backend_serves() {
    let dir = TempDir::new().unwrap();
    let config = write_config(dir.path(), "");

    let out = run(&config, &["item", "delete", "--help"]);
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success(), "{stdout}");

    let footer = "\
Backends:
  item.remove: full on vdir, caldav, gcal, msgraph; partial on pimdir
    pimdir: queued for the next sync, and refused when a source of the store does not support it
  item.remove.if-match: none on vdir; full on pimdir, caldav, gcal; partial on msgraph
    vdir: the backend exposes no version to compare, so `--if-match` is refused
    msgraph: compared with the event's `changeKey` read just before the write, so a write landing in between is not caught
";
    assert!(stdout.ends_with(footer), "{stdout}");

    let out = run(&config, &["item", "build", "--help"]);
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(!stdout.contains("Backends:"), "{stdout}");
}
