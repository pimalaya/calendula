use std::fmt;

use anyhow::Result;
use clap::Parser;
use io_gcal::v3::rest::settings::list::GcalSettingsListParams;
use pimalaya_cli::{
    printer::Printer,
    table::{Cell, Row, Table, TableStyle},
};
use serde::Serialize;

use crate::gcal::client::GcalClient;

/// Show the account's own Calendar settings.
///
/// These are what a Google client renders with (the account's time
/// zone, which day its weeks start on, its date format), and none of
/// them belongs to a calendar, so no shared command can carry them.
///
/// JSON output: `{"settings": [{"id", "value"}]}`.
#[derive(Debug, Parser)]
pub struct GcalSettingsCommand;

impl GcalSettingsCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GcalClient) -> Result<()> {
        let style = client.account.table_style();

        let mut rows = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let params = GcalSettingsListParams {
                page_token: page_token.as_deref(),
                ..Default::default()
            };
            let page = client.settings_list(&params)?.response;

            rows.extend(page.items.into_iter().map(|setting| SettingRow {
                id: setting.id.unwrap_or_default(),
                value: setting.value.unwrap_or_default(),
            }));

            match page.next_page_token {
                Some(next) => page_token = Some(next),
                None => break,
            }
        }

        printer.out(SettingsTable { style, rows })
    }
}

/// The rendered settings listing.
#[derive(Clone, Debug, Serialize)]
pub struct SettingsTable {
    #[serde(skip)]
    pub style: TableStyle,
    #[serde(rename = "settings")]
    pub rows: Vec<SettingRow>,
}

/// One user setting.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct SettingRow {
    /// The setting id, such as `timezone` or `weekStart`.
    pub id: String,
    /// Its value, whose shape depends on the id.
    pub value: String,
}

impl fmt::Display for SettingsTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(self.style)
            .set_header(Row::from([Cell::new("SETTING"), Cell::new("VALUE")]))
            .add_rows(self.rows.iter().map(|setting| {
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(&setting.id))
                    .add_cell(Cell::new(&setting.value));
                row
            }));

        writeln!(f)?;
        writeln!(f, "{table}")
    }
}
