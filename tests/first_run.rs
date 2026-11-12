//! # First run
//!
//! The built binary run without a configuration, stdin a pipe, so it
//! answers as a script sees it: the help for a bare invocation, the
//! missing path and the sample for a command needing an account.

use std::{
    path::Path,
    process::{Command, Output, Stdio},
};

use tempfile::TempDir;

/// The documented sample a missing configuration points at.
const SAMPLE_URL: &str = "https://github.com/pimalaya/calendula/blob/master/config.sample.toml";

/// Runs the binary with `args` in `home`, which holds no configuration.
fn run(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_calendula"))
        .args(args)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .env_remove("XDG_CONFIG_DIRS")
        .env_remove("CALENDULA_CONFIG")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn a_bare_invocation_without_configuration_prints_the_help() {
    let home = TempDir::new().unwrap();
    let path = home.path().join("config.toml");
    let output = run(home.path(), &["-c", path.to_str().unwrap()]);
    let stdout = text(&output.stdout);

    assert!(output.status.success(), "{stdout}");
    assert!(stdout.contains("Usage: calendula"), "{stdout}");
    assert!(!stdout.contains("configure"), "{stdout}");
}

#[test]
fn a_command_without_configuration_names_the_path_and_the_sample() {
    let home = TempDir::new().unwrap();
    let path = home.path().join("config.toml");
    let missing = format!(
        "No configuration found at {}, write one by hand from the sample: {SAMPLE_URL}",
        path.display()
    );

    for command in [
        &["account", "list"][..],
        &["account", "check"],
        &["calendar", "list"],
    ] {
        let args = [&["-c", path.to_str().unwrap()][..], command].concat();
        let output = run(home.path(), &args);
        let stdout = text(&output.stdout);

        assert_eq!(output.status.code(), Some(1), "{command:?}: {stdout}");
        assert!(stdout.contains(&missing), "{command:?}: {stdout}");
    }
}
