//! # Profile get command
//!
//! Reads the signed-in user and renders it.

use core::fmt;

use anyhow::Result;
use clap::Parser;
use io_msgraph::v1::rest::users::MsgraphUser;
use pimalaya_cli::{fields::Fields, printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::msgraph::client::MsgraphClient;

/// GET the signed-in user (Graph `/me`).
///
/// JSON output: the raw Graph user object.
#[derive(Debug, Parser)]
pub struct MsgraphProfileGetCommand;

impl MsgraphProfileGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let user = client.me()?.response;
        printer.out(MsgraphProfileGetOutput(user))
    }
}

/// The signed-in user, emitted verbatim by `--json`.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct MsgraphProfileGetOutput(
    #[schemars(with = "serde_json::Map<String, serde_json::Value>")] pub MsgraphUser,
);

impl fmt::Display for MsgraphProfileGetOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let user = &self.0;
        let text = |value: &Option<String>| match value {
            Some(value) => sanitize::text(value).into_owned(),
            None => String::new(),
        };

        let mut fields = Fields::default();
        fields.push("Id", sanitize::text(&user.id));
        fields.push("Display name", text(&user.display_name));
        fields.push("Mail", text(&user.mail));
        fields.push("User principal name", text(&user.user_principal_name));
        write!(f, "{fields}")
    }
}
