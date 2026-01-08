// This file is part of Calendula, a CLI to manage calendars.
//
// Copyright (C) 2025-2026 soywod <clement.douin@posteo.net>
//
// This program is free software: you can redistribute it and/or
// modify it under the terms of the GNU Affero General Public License
// as published by the Free Software Foundation, either version 3 of
// the License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public
// License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

use std::collections::HashMap;

use pimalaya_toolbox::config::TomlConfig;
use serde::Deserialize;

use crate::account::Account;

/// The main configuration.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The configuration of all the accounts.
    pub accounts: HashMap<String, Account>,
}

// #[cfg(feature = "wizard")]
// impl TomlConfig {
//     pub fn from_wizard(path: &std::path::Path) -> anyhow::Result<Self> {
//         crate::wizard::edit(path, Self::default(), None, Default::default())
//     }
// }

impl TomlConfig for Config {
    type Account = Account;

    fn project_name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn find_default_account(&self) -> Option<(String, Self::Account)> {
        self.accounts
            .iter()
            .find(|(_, account)| account.default)
            .map(|(name, account)| (name.to_owned(), account.clone()))
    }

    fn find_account(&self, name: &str) -> Option<(String, Self::Account)> {
        self.accounts
            .get(name)
            .map(|account| (name.to_owned(), account.clone()))
    }

    // fn from_paths_or_default(paths: &[PathBuf]) -> pimalaya_tui::Result<Self> {
    //     match paths.len() {
    //         0 => Self::from_default_paths(),
    //         _ if paths[0].exists() => Self::from_paths(paths),
    //         #[cfg(feature = "wizard")]
    //         _ => {
    //             use pimalaya_tui::terminal::{print, prompt};

    //             let path = &paths[0];
    //             print::warn(format!("Cannot find configuration at {}.", path.display()));

    //             if !prompt::bool("Would you like to create one with the wizard?", true)? {
    //                 std::process::exit(0);
    //             }

    //             Self::from_wizard(&paths[0])
    //                 .map_err(pimalaya_tui::Error::CreateTomlConfigFromWizardError)
    //         }
    //         #[cfg(not(feature = "wizard"))]
    //         _ => Err(pimalaya_tui::Error::CreateTomlConfigFromInvalidPathsError),
    //     }
    // }
}
