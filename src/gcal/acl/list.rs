//! # ACL list
//!
//! The `calendula gcal acl list` command, listing the sharing rules of one
//! calendar.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use io_gcal::v3::rest::acl::list::GcalAclListParams;
use pimalaya_cli::{columns::Columns, printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::gcal::{client::GcalClient, render::str_of};

/// List the access control rules of a calendar.
///
/// Each rule grants one scope one role. The id a rule shows here is
/// what `acl delete` takes.
///
/// JSON output: `{"rules": [<ACL rule>...]}`, every rule of every page as
/// Google sent it (`id`, `scope: {type, value}`, `role` and the rest).
#[derive(Debug, Parser)]
pub struct GcalAclListCommand {
    /// The calendar whose rules are listed. Falls back to
    /// `calendar.default`.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR-ID")]
    pub calendar_id: Option<String>,
}

impl GcalAclListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GcalClient) -> Result<()> {
        let calendar_id = client.account.calendar_id(self.calendar_id)?;
        let params = GcalAclListParams::default();
        let rules = client.raw_items(&["calendars", &calendar_id, "acl"], &params)?;

        printer.out(GcalAclListOutput { rules })
    }
}

/// The ACL listing.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GcalAclListOutput {
    /// The rules of the calendar, each as Google sent it.
    #[schemars(with = "Vec<Map<String, Value>>")]
    pub rules: Vec<Value>,
}

impl fmt::Display for GcalAclListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut columns = Columns::new(["ID", "SCOPE", "ROLE", "VALUE"]);

        for rule in &self.rules {
            let scope = rule.get("scope").unwrap_or(&Value::Null);

            columns.push([
                sanitize::text(str_of(rule, "id")).into_owned(),
                sanitize::text(str_of(scope, "type")).into_owned(),
                sanitize::text(str_of(rule, "role")).into_owned(),
                sanitize::text(str_of(scope, "value")).into_owned(),
            ]);
        }

        write!(f, "{columns}")
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// The rules print as Google sent them, the scope nested as Google
    /// nests it, the text one column each.
    #[test]
    fn the_rules_print_as_google_sent_them() {
        let rule = json!({
            "kind": "calendar#aclRule",
            "etag": "\"e1\"",
            "id": "user:bob@example.org",
            "scope": {"type": "user", "value": "bob@example.org"},
            "role": "reader",
        });
        let output = GcalAclListOutput {
            rules: vec![rule.clone()],
        };

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json, json!({"rules": [rule]}));

        let text = output.to_string();
        assert!(text.contains("user:bob@example.org"), "{text}");
        assert!(text.contains("reader"), "{text}");
    }
}
