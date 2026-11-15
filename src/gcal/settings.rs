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
use serde_json::{Map, Value};

use crate::gcal::{client::GcalClient, render::str_of};

/// Show the account's own Calendar settings.
///
/// These are what a Google client renders with: the account's time zone,
/// which day its weeks start on, its date format. None belongs to a
/// calendar, so no unified command can carry them.
///
/// JSON output: `{"settings": [<setting>...]}`, every setting of every page
/// as Google sent it (`kind`, `etag`, `id`, `value`).
#[derive(Debug, Parser)]
pub struct GcalSettingsCommand;

impl GcalSettingsCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GcalClient) -> Result<()> {
        let params = GcalSettingsListParams::default();
        let settings = client.raw_items(&["users", "me", "settings"], &params)?;

        printer.out(GcalSettingsOutput { settings })
    }
}

/// The account settings, a record of one field each.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GcalSettingsOutput {
    /// The account settings, each as Google sent it.
    #[schemars(with = "Vec<Map<String, Value>>")]
    pub settings: Vec<Value>,
}

impl fmt::Display for GcalSettingsOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ids: Vec<String> = self
            .settings
            .iter()
            .map(|setting| sanitize::text(str_of(setting, "id")).into_owned())
            .collect();

        let mut fields = Fields::default();

        for (id, setting) in ids.iter().zip(&self.settings) {
            fields.push(id, sanitize::text(str_of(setting, "value")));
        }

        write!(f, "{fields}")
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// The settings print as Google sent them, `kind` and `etag` kept,
    /// the text one field each.
    #[test]
    fn the_settings_print_as_google_sent_them() {
        let setting = json!({
            "kind": "calendar#setting",
            "etag": "\"e1\"",
            "id": "timezone",
            "value": "Europe/Paris",
        });
        let output = GcalSettingsOutput {
            settings: vec![setting.clone()],
        };

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json, json!({"settings": [setting]}));
        assert_eq!(output.to_string(), "timezone:  Europe/Paris\n");
    }
}
