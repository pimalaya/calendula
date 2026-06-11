use std::path::PathBuf;

use anyhow::{Result, anyhow};
use clap::Parser;
use pimalaya_cli::printer::Printer;
use pimalaya_config::toml::TomlConfig;

use crate::{config::Config, wizard};

/// Edit (or create) an account through the wizard.
///
/// The account is picked with the global `--account` flag (a new entry
/// is created when no account with that name exists); without it, the
/// default account is edited. Runs the CalDAV / vdir wizard with the
/// account's current values as defaults.
///
/// JSON output: none; the command is an interactive wizard only.
#[derive(Debug, Parser)]
pub struct AccountConfigureCommand;

impl AccountConfigureCommand {
    pub fn execute(
        self,
        _printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
    ) -> Result<()> {
        let target = Config::target_path(config_paths)?;
        let config = Config::from_paths_or_default(config_paths)?.unwrap_or_default();

        let name = match account_name {
            Some(name) => name.to_owned(),
            None => default_account_name(&config).ok_or_else(|| {
                anyhow!("Pass --account <name> to choose which account to configure")
            })?,
        };

        wizard::edit::edit_account(&target, config, &name)?;

        Ok(())
    }
}

/// Name of the account flagged `default = true`, if any.
fn default_account_name(config: &Config) -> Option<String> {
    config
        .accounts
        .iter()
        .find_map(|(name, account)| account.default.then(|| name.clone()))
}
