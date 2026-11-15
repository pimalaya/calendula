//! # Raw request command
//!
//! Sends an arbitrary Microsoft Graph request and prints the JSON response,
//! the escape hatch for the endpoints no command covers.

use anyhow::{Context, Result, bail};
use clap::Parser;
use io_msgraph::v1::send::{MSGRAPH_API_BASE, MsgraphSend, MsgraphSendOutput};
use pimalaya_cli::printer::Printer;
use serde_json::Value;
use url::Url;

use crate::{msgraph::client::MsgraphClient, shared::raw_json::RawJsonOutput};

/// Send a raw Microsoft Graph request and print the JSON response.
///
/// The path is joined onto the Graph `v1.0/` base, or a full URL is used
/// as-is. JSON output: `{response}`, the raw Graph response.
#[derive(Debug, Parser)]
pub struct MsgraphRequestCommand {
    /// HTTP method: get, post, patch, delete.
    #[arg(value_name = "METHOD")]
    pub method: String,
    /// Graph path (joined onto `v1.0/`) or a full URL.
    #[arg(value_name = "PATH")]
    pub path: String,
    /// Raw JSON body for post / patch.
    #[arg(value_name = "JSON")]
    pub body: Option<String>,
}

impl MsgraphRequestCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let url = resolve_url(&self.path)?;
        let auth = client.auth.clone();

        let send: MsgraphSend<Value> = match self.method.to_ascii_uppercase().as_str() {
            "GET" => MsgraphSend::get(&auth, url),
            "DELETE" => MsgraphSend::delete(&auth, url),
            "POST" => MsgraphSend::post_json(&auth, url, &parse_body(self.body.as_deref())?)?,
            "PATCH" => MsgraphSend::patch_json(&auth, url, &parse_body(self.body.as_deref())?)?,
            other => bail!("Unsupported method `{other}` (use get/post/patch/delete)"),
        };
        let out: MsgraphSendOutput<Value> = client.run(send)?;

        printer.out(RawJsonOutput {
            response: out.response,
        })
    }
}

/// Parses the raw body argument, an absent body meaning JSON `null`.
fn parse_body(body: Option<&str>) -> Result<Value> {
    serde_json::from_str(body.unwrap_or("null")).context("Parse request body JSON error")
}

/// Resolves a Graph path onto the `v1.0/` base, a full URL being taken
/// as-is.
fn resolve_url(path: &str) -> Result<Url> {
    if path.starts_with("http") {
        Ok(Url::parse(path)?)
    } else {
        Ok(Url::parse(MSGRAPH_API_BASE)?.join(path.trim_start_matches('/'))?)
    }
}
