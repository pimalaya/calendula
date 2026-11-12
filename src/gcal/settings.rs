//! # Settings
//!
//! The `calendula gcal settings` command, showing the account's own Calendar
//! settings.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use io_gcal::v3::rest::settings::list::GcalSettingsListParams;
use pimalaya_cli::{fields::Fields, printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::gcal::client::GcalClient;

/// Show the account's own Calendar settings.
///
/// These are what a Google client renders with: the account's time zone,
/// which day its weeks start on, its date format. None belongs to a
/// calendar, so no unified command can carry them.
///
/// JSON output: `{"settings": [{"id", "value"}]}`.
#[derive(Debug, Parser)]
pub struct GcalSettingsCommand;

impl GcalSettingsCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GcalClient) -> Result<()> {
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

        printer.out(GcalSettingsOutput { rows })
    }
}

/// The account settings, a record of one field each.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GcalSettingsOutput {
    /// The account settings, one entry each.
    #[serde(rename = "settings")]
    pub rows: Vec<SettingRow>,
}

/// One user setting.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SettingRow {
    /// The setting id, such as `timezone` or `weekStart`.
    pub id: String,
    /// Its value, whose shape depends on the id.
    pub value: String,
}

impl fmt::Display for GcalSettingsOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ids: Vec<String> = self
            .rows
            .iter()
            .map(|setting| sanitize::text(&setting.id).into_owned())
            .collect();

        let mut fields = Fields::default();

        for (id, setting) in ids.iter().zip(&self.rows) {
            fields.push(id, sanitize::text(&setting.value));
        }

        write!(f, "{fields}")
    }
}
