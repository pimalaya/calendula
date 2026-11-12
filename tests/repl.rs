//! # REPL
//!
//! The built binary's `repl` over a vdir account, fed a piped session:
//! every line answers once and the session survives its failures, a
//! line's `--json`, a `#` comment, a session flag and the help it asks for
//! are answered as pimalaya-cli's shared loop says, and what cannot run
//! inside a session (a session, the composer, stdin) is refused by name
//! while the session goes on.

#![cfg(feature = "vdir")]

use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use serde_json::Value;
use tempfile::TempDir;

const EVENT: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\n\
                     BEGIN:VEVENT\r\nUID:standup@example.org\r\n\
                     DTSTAMP:20261101T000000Z\r\nDTSTART:20261112T090000Z\r\n\
                     DTEND:20261112T093000Z\r\nSUMMARY:Standup\r\nEND:VEVENT\r\n\
                     END:VCALENDAR\r\n";

/// A home holding a vdir calendar `personal` with the event `standup`,
/// and the configuration of an account reading it.
fn account() -> TempDir {
    let home = TempDir::new().unwrap();
    let calendar = home.path().join("calendars").join("personal");
    fs::create_dir_all(&calendar).unwrap();
    fs::write(calendar.join("standup.ics"), EVENT).unwrap();

    let calendars = home.path().join("calendars");
    let toml = format!(
        "[accounts.perso]\ndefault = true\ncalendar.default = \"personal\"\n\
         vdir.home-dir = {:?}\n",
        calendars.display().to_string(),
    );
    fs::write(home.path().join("config.toml"), toml).unwrap();

    home
}

/// Runs a piped session started with `flags` over `lines` in `home`, and
/// returns its stdout and stderr.
fn run(home: &Path, flags: &[&str], lines: &str) -> (String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_calendula"))
        .arg("-c")
        .arg(home.join("config.toml"))
        .args(flags)
        .arg("repl")
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .env_remove("XDG_CONFIG_DIRS")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .take()
        .unwrap()
        .write_all(lines.as_bytes())
        .unwrap();

    let output = child.wait_with_output().unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(output.status.success(), "{stderr}");

    (String::from_utf8(output.stdout).unwrap(), stderr)
}

/// The JSON values of a session's stdout, one per line.
fn values(stdout: &str) -> Vec<Value> {
    stdout
        .lines()
        .map(|line| serde_json::from_str(line).unwrap_or_else(|err| panic!("{err}: {line}")))
        .collect()
}

/// Runs a piped `--json` session over `lines`, one JSON value per answer.
fn session(home: &Path, lines: &str) -> Vec<Value> {
    values(&run(home, &["--json"], lines).0)
}

#[test]
fn a_session_answers_each_line_and_survives_its_failures() {
    let home = account();
    let answers = session(
        home.path(),
        "calendar list\n\
         nope\n\
         event read nope\n\
         event list \"unclosed\n\
         event list\n\
         quit\n\
         calendar list\n",
    );

    assert_eq!(answers.len(), 5, "{answers:#?}");
    assert_eq!(answers[0]["calendars"][0]["id"], "personal");
    assert!(answers[1]["error"].is_string(), "{}", answers[1]);
    assert!(answers[2]["error"].is_string(), "{}", answers[2]);
    assert!(answers[3]["error"].is_string(), "{}", answers[3]);
    assert_eq!(answers[4]["events"][0]["summary"], "Standup");
}

#[test]
fn a_line_json_flag_switches_that_line_alone() {
    let home = account();
    let (stdout, _) = run(
        home.path(),
        &[],
        "event list --json\n\
         calendar list\n",
    );
    let mut lines = stdout.lines();

    let json: Value = serde_json::from_str(lines.next().unwrap()).unwrap();
    assert_eq!(json["events"][0]["id"], "standup", "{stdout}");

    let text: Vec<&str> = lines.collect();
    assert!(
        text.iter().any(|line| line.starts_with("personal")),
        "{stdout}"
    );
    assert!(!text.iter().any(|line| line.starts_with('{')), "{stdout}");
}

#[test]
fn a_session_flag_is_refused_by_name() {
    let home = account();
    let answers = session(
        home.path(),
        "event list -a other\n\
         event list -b vdir\n\
         event list --backend=caldav\n\
         event list -c other.toml\n",
    );

    assert_eq!(answers.len(), 4, "{answers:#?}");

    for (answer, flag) in answers.iter().zip(["-a", "-b", "--backend", "-c"]) {
        let message =
            format!("Cannot set `{flag}` inside a session, start a new session with it instead");
        assert_eq!(answer["error"], message, "{answer}");
    }
}

#[test]
fn a_comment_line_is_skipped() {
    let home = account();
    let answers = session(
        home.path(),
        "# calendar list\n\
         \x20  # event delete standup\n\
         event list # every one\n",
    );

    assert_eq!(answers.len(), 1, "{answers:#?}");
    assert_eq!(answers[0]["events"][0]["id"], "standup");
}

#[test]
fn help_goes_to_stderr_in_a_json_session() {
    let home = account();
    let (stdout, stderr) = run(
        home.path(),
        &["--json"],
        "help\n\
         event list --help\n\
         event list\n",
    );

    let answers = values(&stdout);
    assert_eq!(answers.len(), 1, "{stdout}");
    assert_eq!(answers[0]["events"][0]["id"], "standup");

    let usage = "Usage: calendula [OPTIONS] <COMMAND>";
    assert!(stderr.contains(usage), "{stderr}");
    let usage = "Usage: calendula event list [OPTIONS]";
    assert!(stderr.contains(usage), "{stderr}");
}

#[test]
fn what_cannot_run_in_a_session_is_refused_and_the_session_goes_on() {
    let home = account();
    let answers = session(
        home.path(),
        "repl\n\
         mcp\n\
         event create -i\n\
         item update -i --composer true standup\n\
         event build -i\n\
         event create -\n\
         item update standup -\n\
         event list\n",
    );

    assert_eq!(answers.len(), 8, "{answers:#?}");

    for answer in &answers[..2] {
        let error = answer["error"].as_str().unwrap_or_default();
        assert_eq!(error, "Cannot start a session inside a session", "{answer}");
    }

    for answer in &answers[2..5] {
        let error = answer["error"].as_str().unwrap_or_default();
        assert!(error.contains("composer (-i)"), "{answer}");
    }

    assert_eq!(answers[5]["code"], "stdin-taken", "{}", answers[5]);
    assert_eq!(answers[6]["code"], "stdin-taken", "{}", answers[6]);

    let events = answers[7]["events"].as_array().unwrap();
    assert_eq!(events.len(), 1, "{events:?}");
    assert_eq!(events[0]["summary"], "Standup");
}
