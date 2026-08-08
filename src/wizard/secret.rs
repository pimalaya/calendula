//! Secret prompts for the wizard.
//!
//! Delegates to pimalaya-cli's OS-aware pickers: [`configure_password`]
//! offers the OS keyrings, [`configure_token`] the OAuth 2.0 token
//! brokers (Ortie, pizauth, oama). Both also allow a custom command or
//! a raw value. A known provider or broker yields an argv command,
//! which serializes as a TOML array; a custom command is a shell
//! string.
//!
//! calendula only reads a secret: the value must already be stored, and
//! a missing one surfaces when the account is tested right after.

use std::process::Command;

use anyhow::{Result, bail};
use pimalaya_cli::wizard::keyring::{self, SecretChoice};
use pimalaya_config::{command::shell, secret::Secret};

/// Prompts for a password through the shared keyring picker.
///
/// `key_default` seeds the keyring entry, typically
/// `<account>-caldav`. The entry is used verbatim, so a pre-existing
/// secret is read exactly as named.
pub fn configure_password(label: &str, key_default: &str) -> Result<Secret> {
    to_secret(keyring::prompt_secret(label, key_default)?)
}

/// Prompts for an API token through the shared token picker, which
/// combines the OS keyrings (for a token generated on the provider)
/// with the OAuth 2.0 brokers when `oauth` is true. A broker refreshes
/// and prints a fresh token on every read.
pub fn configure_token(label: &str, key_default: &str, oauth: bool) -> Result<Secret> {
    to_secret(keyring::prompt_token(label, key_default, oauth)?)
}

fn to_secret(choice: SecretChoice) -> Result<Secret> {
    Ok(match choice {
        SecretChoice::Command(argv) => command_secret(argv)?,
        SecretChoice::Shell(line) => shell_secret(&line)?,
        SecretChoice::Raw(secret) => Secret::Raw(secret),
    })
}

/// Builds a command secret from an argv (a program plus arguments, no
/// shell), the form a known keyring provider or token broker yields. It
/// serializes back as a TOML array.
fn command_secret(argv: Vec<String>) -> Result<Secret> {
    let Some((program, args)) = argv.split_first() else {
        bail!("Empty command for secret");
    };

    let mut command = Command::new(program);
    command.args(args);

    Ok(Secret::Command(command))
}

/// Builds a command secret from a shell command line, the form a user
/// typed by hand. It serializes back as a TOML string.
fn shell_secret(line: &str) -> Result<Secret> {
    let line = line.trim();

    if line.is_empty() {
        bail!("Empty shell command for secret");
    }

    Ok(Secret::Command(shell(line)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_command_is_rejected_rather_than_stored() {
        assert!(command_secret(Vec::new()).is_err());
        assert!(shell_secret("   ").is_err());
    }
}
