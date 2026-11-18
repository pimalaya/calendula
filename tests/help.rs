//! # Help
//!
//! The built binary's help and its refusal of a `--backend` it does not
//! know, worded as Himalaya words them: the `-b` help, the possible values
//! a refused `-b` lists, the `account` abouts and the namespace abouts.

use std::{
    path::Path,
    process::{Command, Output, Stdio},
};

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

/// The help `args` print, on stdout.
fn help(args: &[&str]) -> String {
    let home = TempDir::new().unwrap();
    let output = run(home.path(), args);
    assert!(output.status.success(), "{args:?}");
    String::from_utf8(output.stdout).unwrap()
}

/// A `--backend` none is spelled is refused listing every backend
/// calendula knows, built in or not, as Himalaya's and carillon's are.
#[test]
fn an_unknown_backend_lists_the_possible_values() {
    let home = TempDir::new().unwrap();
    let output = run(home.path(), &["-b", "nope", "account", "list"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(
        stderr.contains("[possible values: auto, vdir, pimdir, caldav, jmap, gcal, msgraph]"),
        "{stderr}"
    );
}

/// A backend the build leaves out is still refused naming its feature.
#[cfg(not(feature = "gcal"))]
#[test]
fn a_backend_left_out_names_its_feature() {
    let home = TempDir::new().unwrap();
    let output = run(home.path(), &["-b", "gcal", "account", "list"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(
        stderr.contains("rebuild with `--features gcal`"),
        "{stderr}"
    );
}

#[test]
fn the_backend_help_reads_as_himalayas() {
    let help = help(&["--help"]);

    assert!(
        help.contains("Force a specific backend for cross-protocol commands."),
        "{help}"
    );
    assert!(
        help.contains("- caldav:  Pin the command to the account's CalDAV backend"),
        "{help}"
    );
}

#[test]
fn the_abouts_read_as_himalayas() {
    let top = help(&["--help"]);

    // NOTE: a namespace is listed only in a build carrying its backend.
    let abouts = [
        (
            true,
            "account      Manage accounts defined in the TOML configuration file",
        ),
        (cfg!(feature = "caldav"), "caldav       CalDAV-specific API"),
        (
            cfg!(feature = "msgraph"),
            "msgraph      Microsoft Graph-specific API",
        ),
        (
            cfg!(feature = "gcal"),
            "gcal         Google Calendar-specific API",
        ),
        (cfg!(feature = "pimdir"), "pimdir       pimdir-specific API"),
        (cfg!(feature = "vdir"), "vdir         vdir-specific API"),
    ];

    for (built, about) in abouts {
        assert_eq!(top.contains(about), built, "{about}: {top}");
    }

    let account = help(&["account", "--help"]);
    let list = "list        List all accounts declared in the configuration";
    assert!(account.contains(list), "{account}");
}
