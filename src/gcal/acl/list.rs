use std::fmt;

use anyhow::Result;
use clap::Parser;
use io_gcal::v3::rest::acl::list::GcalAclListParams;
use pimalaya_cli::{
    printer::Printer,
    table::{Cell, Row, Table, TableStyle},
};
use serde::Serialize;

use crate::gcal::{client::GcalClient, render};

/// List the access control rules of a calendar.
///
/// Each rule grants one scope one role. The id a rule shows here is
/// what `acl delete` takes.
///
/// JSON output: `{"rules": [{"id", "scope", "value", "role"}]}`.
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
        let style = client.account.table_style();

        let mut rows = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let params = GcalAclListParams {
                page_token: page_token.as_deref(),
                ..Default::default()
            };
            let page = client.acl_list(&calendar_id, &params)?.response;

            rows.extend(page.items.into_iter().map(|rule| {
                RuleRow {
                    id: rule.id.unwrap_or_default(),
                    scope: rule
                        .scope
                        .as_ref()
                        .and_then(|scope| scope.scope_type)
                        .map(render::scope_type),
                    value: rule.scope.and_then(|scope| scope.value),
                    role: rule.role.map(render::access_role),
                }
            }));

            match page.next_page_token {
                Some(next) => page_token = Some(next),
                None => break,
            }
        }

        printer.out(RulesTable { style, rows })
    }
}

/// The rendered ACL listing.
#[derive(Clone, Debug, Serialize)]
pub struct RulesTable {
    #[serde(skip)]
    pub style: TableStyle,
    #[serde(rename = "rules")]
    pub rows: Vec<RuleRow>,
}

/// One access control rule.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct RuleRow {
    /// The rule identifier, `<scope type>:<scope value>`.
    pub id: String,
    /// The kind of grantee: `default`, `user`, `group` or `domain`.
    pub scope: Option<&'static str>,
    /// The address or domain the scope names, absent for `default`.
    pub value: Option<String>,
    /// The role the rule grants.
    pub role: Option<&'static str>,
}

impl fmt::Display for RulesTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(self.style)
            .set_header(Row::from([
                Cell::new("ID"),
                Cell::new("SCOPE"),
                Cell::new("VALUE"),
                Cell::new("ROLE"),
            ]))
            .add_rows(self.rows.iter().map(|rule| {
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(&rule.id))
                    .add_cell(Cell::new(rule.scope.unwrap_or("")))
                    .add_cell(Cell::new(rule.value.as_deref().unwrap_or("")))
                    .add_cell(Cell::new(rule.role.unwrap_or("")));
                row
            }));

        writeln!(f)?;
        writeln!(f, "{table}")
    }
}
