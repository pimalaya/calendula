//! Interactive configuration wizard.
//!
//! Triggered by `cli::load_or_wizard` when no config file is found.
//!
//! Flow:
//!
//! 1. Confirm with the user. Exit if they decline.
//! 2. Ask for an account name.
//! 3. Ask whether the account is backed by vdir or caldav.
//! 4. Run [`crate::wizard::edit::edit_account`] to gather the rest of
//!    the account fields.

use std::{collections::HashMap, path::Path, process::exit};

use anyhow::Result;
use log::info;
use pimalaya_cli::prompt;

use crate::{config::Config, wizard::edit};

pub fn run_or_exit(target: &Path) -> Result<Config> {
    let prompt_msg = format!(
        "No configuration found. Create one at {}?",
        target.display(),
    );

    if !prompt::bool(&prompt_msg, true)? {
        exit(0);
    }

    let account_name = prompt::text("Account name:", Some("default"))?;
    let mut config = Config {
        accounts: HashMap::new(),
        ..Default::default()
    };

    config = edit::edit_account(target, config, &account_name)?;
    info!("Configuration written to {}", target.display());

    Ok(config)
}
