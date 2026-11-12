//! # Account list
//!
//! The `account list` command, listing every account the configuration
//! declares.

use std::{fmt, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{columns::Columns, printer::Printer, sanitize};
use pimalaya_config::toml::TomlConfig;
use schemars::JsonSchema;
use serde::Serialize;

use crate::config::{AccountConfig, Config};

/// List the accounts declared in the configuration.
///
/// Each row shows the account name, the backends it carries a
/// configuration block for, and whether it is the default account.
///
/// JSON output: `{"accounts": [{"name", "default", "backends"}]}`.
#[derive(Debug, Parser)]
pub struct AccountListCommand;

impl AccountListCommand {
    /// Loads the configuration, taking no account, and prints its
    /// accounts as a list.
    pub fn execute(self, printer: &mut impl Printer, config_paths: &[PathBuf]) -> Result<()> {
        let config = Config::load(config_paths)?;

        let mut accounts: Vec<AccountRow> = config
            .accounts
            .iter()
            .map(|(name, account)| AccountRow::from_account(name, account))
            .collect();
        accounts.sort_by(|a, b| a.name.cmp(&b.name));

        printer.out(AccountListOutput { accounts })
    }
}

/// One account's row in an [`AccountListOutput`].
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccountRow {
    /// The `[accounts.<name>]` table key.
    pub name: String,
    /// Whether the account is flagged `default = true`.
    pub default: bool,
    /// The backends the account carries a configuration block for.
    pub backends: Vec<&'static str>,
}

impl AccountRow {
    fn from_account(name: &str, account: &AccountConfig) -> Self {
        #[cfg_attr(
            not(any(
                feature = "vdir",
                feature = "caldav",
                feature = "pimdir",
                feature = "gcal",
                feature = "msgraph"
            )),
            allow(unused_mut)
        )]
        let mut backends = Vec::new();

        #[cfg(feature = "vdir")]
        if account.vdir.is_some() {
            backends.push("vdir");
        }

        #[cfg(feature = "pimdir")]
        if account.pimdir.is_some() {
            backends.push("pimdir");
        }

        #[cfg(feature = "caldav")]
        if account.caldav.is_some() {
            backends.push("caldav");
        }

        #[cfg(feature = "gcal")]
        if account.gcal.is_some() {
            backends.push("gcal");
        }
        #[cfg(feature = "msgraph")]
        if account.msgraph.is_some() {
            backends.push("msgraph");
        }

        Self {
            name: name.to_owned(),
            default: account.default,
            backends,
        }
    }
}

/// The account listing.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccountListOutput {
    /// One row per account, sorted by name.
    pub accounts: Vec<AccountRow>,
}

impl fmt::Display for AccountListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut columns = Columns::new(["NAME", "BACKENDS", "DEFAULT"]);

        for account in &self.accounts {
            columns.push([
                sanitize::text(&account.name).into_owned(),
                account.backends.join(","),
                String::from(if account.default { "yes" } else { "" }),
            ]);
        }

        write!(f, "{columns}")
    }
}

#[cfg(test)]
mod tests {
    use std::{env, process};

    use pimalaya_cli::{clap::args::JsonFlag, printer::StdoutPrinter};
    use pimalaya_config::toml::ConfigMissingError;

    use crate::account::list::{AccountListCommand, AccountListOutput, AccountRow};

    /// Listing takes no account, but a missing configuration still names
    /// the path it looked at.
    #[test]
    fn a_missing_configuration_names_the_path() {
        let dir = env::temp_dir().join(format!("calendula-account-list-{}", process::id()));
        let paths = [dir.join("config.toml")];
        let mut printer = StdoutPrinter::new(&JsonFlag::default());

        let err = AccountListCommand
            .execute(&mut printer, &paths)
            .unwrap_err();

        let missing = err.downcast_ref::<ConfigMissingError>().unwrap();
        assert_eq!(missing.path, paths[0]);
        assert!(err.to_string().contains(&paths[0].display().to_string()));
    }

    #[test]
    fn accounts_print_as_columns() {
        let row = |name: &str, default, backends| AccountRow {
            name: name.into(),
            default,
            backends,
        };
        let output = AccountListOutput {
            accounts: vec![
                row("perso", true, vec!["vdir", "caldav"]),
                row("work", false, vec!["gcal"]),
            ],
        };

        let expected = "\
NAME   BACKENDS     DEFAULT
perso  vdir,caldav  yes
work   gcal         -
";
        assert_eq!(output.to_string(), expected);
    }
}
