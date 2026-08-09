use std::{fmt, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use crossterm::style::Color as CrosstermColor;
use pimalaya_cli::{
    printer::Printer,
    table::{Cell, Color, ContentArrangement, Row, Table, TableStyle},
};
use pimalaya_config::toml::TomlConfig;
use serde::Serialize;

use crate::{
    account::context::map_color_or,
    config::{AccountConfig, Config, TableArrangementConfig},
    shared::table::{DEFAULT_PRESET, style_from_preset},
};

/// List the accounts declared in the configuration.
///
/// Each row shows the account name, the backends it carries a
/// configuration block for, and whether it is the default account.
///
/// JSON output: `{"accounts": [{"name", "default", "backends"}]}`.
#[derive(Debug, Parser)]
pub struct AccountListCommand;

impl AccountListCommand {
    pub fn execute(self, printer: &mut impl Printer, config_paths: &[PathBuf]) -> Result<()> {
        let config = load_config(config_paths)?;

        let style = style_from_preset(config.table.preset.as_deref().unwrap_or(DEFAULT_PRESET));
        let arrangement = config
            .table
            .arrangement
            .clone()
            .unwrap_or(TableArrangementConfig::Dynamic)
            .into();

        let table_config = &config.account.list.table;
        let colors = AccountColors {
            name: map_color_or(table_config.name_color, CrosstermColor::Green),
            backends: map_color_or(table_config.backends_color, CrosstermColor::Blue),
            default: map_color_or(table_config.default_color, CrosstermColor::Reset),
        };

        let mut accounts: Vec<AccountRow> = config
            .accounts
            .iter()
            .map(|(name, account)| AccountRow::from_account(name, account))
            .collect();
        accounts.sort_by(|a, b| a.name.cmp(&b.name));

        printer.out(AccountsTable {
            style,
            arrangement,
            colors,
            accounts,
        })
    }
}

/// The per-column colors an account listing renders with.
#[derive(Clone, Copy, Debug)]
struct AccountColors {
    name: Color,
    backends: Color,
    default: Color,
}

/// Loads the configuration, or explains where one comes from.
fn load_config(paths: &[PathBuf]) -> Result<Config> {
    match Config::from_paths_or_default(paths)? {
        Some(config) => Ok(config),
        None => anyhow::bail!(
            "No configuration found. Run `calendula` to generate one with the wizard."
        ),
    }
}

/// One account's row in an [`AccountsTable`].
#[derive(Clone, Debug, Serialize)]
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

        Self {
            name: name.to_owned(),
            default: account.default,
            backends,
        }
    }
}

/// The rendered account listing.
#[derive(Clone, Debug, Serialize)]
pub struct AccountsTable {
    #[serde(skip)]
    pub style: TableStyle,
    #[serde(skip)]
    pub arrangement: ContentArrangement,
    #[serde(skip)]
    colors: AccountColors,
    pub accounts: Vec<AccountRow>,
}

impl fmt::Display for AccountsTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(self.style)
            .set_content_arrangement(self.arrangement.clone())
            .set_header(Row::from(vec![
                Cell::new("NAME"),
                Cell::new("BACKENDS"),
                Cell::new("DEFAULT"),
            ]))
            .add_rows(self.accounts.iter().map(|account| {
                let mut row = Row::new();
                row.max_height(1);
                row.add_cell(Cell::new(&account.name).fg(self.colors.name));
                row.add_cell(Cell::new(account.backends.join(", ")).fg(self.colors.backends));
                row.add_cell(
                    Cell::new(if account.default { "yes" } else { "" }).fg(self.colors.default),
                );
                row
            }));

        writeln!(f)?;
        writeln!(f, "{table}")
    }
}
