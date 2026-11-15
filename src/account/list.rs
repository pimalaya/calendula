//! # Account list
//!
//! The `account list` command, listing every account the configuration
//! declares, as pimalaya-cli's shared [`AccountListOutput`] prints it.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{
    account::{AccountListOutput, AccountRow},
    printer::Printer,
};
use pimalaya_config::toml::TomlConfig;

use crate::config::Config;

/// List the accounts declared in the configuration.
///
/// One line per account: its name, the backends it carries a
/// configuration block for, and `default` for the account a command with
/// no `-a` runs against. A backend this build leaves out shows as
/// `<name> (not built)`, its block loading unread.
///
/// JSON output: `{"accounts": [{"name", "default", "backends",
/// "notBuilt"}]}`, `backends` the ones this build serves.
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
            .map(|(name, account)| AccountRow {
                name: name.to_owned(),
                default: account.default,
                backends: account.backends(),
                not_built: account.not_built(),
            })
            .collect();
        accounts.sort_by(|a, b| a.name.cmp(&b.name));

        printer.out(AccountListOutput { accounts })
    }
}

#[cfg(test)]
mod tests {
    use std::{env, process};

    use pimalaya_cli::{clap::args::JsonFlag, printer::StdoutPrinter};
    use pimalaya_config::toml::ConfigMissingError;

    use crate::account::list::AccountListCommand;

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
}
