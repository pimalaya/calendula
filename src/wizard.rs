// This file is part of Calendula, a CLI to manage calendars.
//
// Copyright (C) 2025 soywod <clement.douin@posteo.net>
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

use std::{fmt, path::Path};

use anyhow::Result;
use pimalaya_tui::terminal::{print, prompt};

use crate::{
    account::config::{Backend, TomlAccountConfig},
    config::TomlConfig,
};

#[derive(Eq, PartialEq)]
pub enum BackendKind {
    Caldav,
    Vdir,
}

impl fmt::Display for BackendKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Caldav => write!(f, "Caldav"),
            Self::Vdir => write!(f, "Vdir"),
        }
    }
}

impl BackendKind {
    pub const ALL: [Self; 2] = [Self::Caldav, Self::Vdir];
}

pub fn edit(
    path: impl AsRef<Path>,
    mut config: TomlConfig,
    account_name: Option<&str>,
    mut account_config: TomlAccountConfig,
) -> Result<TomlConfig> {
    todo!()

    // match account_name.as_ref() {
    //     Some(name) => print::section(format!("Configuring your account {name}")),
    //     None => print::section("Configuring your default account"),
    // };

    // let default =
    //     account_name.is_none() || prompt::bool("Should this account be the default one?", false)?;

    // if default {
    //     config
    //         .accounts
    //         .iter_mut()
    //         .for_each(|(_, config)| config.default = false)
    // }

    // account_config.default = default;

    // let account_name = prompt::text("Account name:", None)?;

    // let backend = prompt::item("Backend:", &BackendKind::ALL, None)?;

    // match backend {
    //     #[cfg(feature = "caldav")]
    //     BackendKind::Caldav => {
    //         account_config.backend = Backend::Caldav(config);
    //     }
    //     #[cfg(not(feature = "caldav"))]
    //     BackendKind::Caldav => {
    //         bail!("Missing cargo feature `caldav`, `caldav-native-tls` or `caldav-rustls`");
    //     }
    //     #[cfg(feature = "vdir")]
    //     BackendKind::Vdir => {
    //         account_config.backend = Backend::Vdir(config);
    //     }
    //     #[cfg(not(feature = "vdir"))]
    //     BackendKind::Vdir => {
    //         bail!("Missing cargo feature `vdir`");
    //     }
    // }

    // config.accounts.insert(account_name, account_config);
    // config.write(path.as_ref())?;

    // Ok(config)
}
