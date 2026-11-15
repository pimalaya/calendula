//! # Raw request command
//!
//! Sends an arbitrary Google Calendar API request and prints the JSON
//! response, the escape hatch for the endpoints no command covers.

use anyhow::{Context, Result, bail};
use clap::Parser;
use io_gcal::v3::send::{GCAL_API_BASE, GcalSend, GcalSendOutput};
use pimalaya_cli::printer::Printer;
use serde_json::Value;
use url::Url;

use crate::{
    gcal::client::GcalClient,
    shared::raw_json::{RawJsonOutput, request_url},
};

/// Send a raw Google Calendar API request and print the JSON response.
///
/// The path is joined onto the Calendar `v3/` base, or a full URL is used
/// as-is when it is on Google's API origin, refused before anything is
/// sent otherwise. JSON output: `{response}`, the raw Calendar response.
#[derive(Debug, Parser)]
pub struct GcalRequestCommand {
    /// HTTP method: get, post, put, patch, delete.
    #[arg(value_name = "METHOD")]
    pub method: String,
    /// Calendar path (joined onto `v3/`) or a full URL.
    #[arg(value_name = "PATH")]
    pub path: String,
    /// Raw JSON body for post / put / patch.
    #[arg(value_name = "JSON")]
    pub body: Option<String>,
}

impl GcalRequestCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GcalClient) -> Result<()> {
        let url = resolve_url(&self.path)?;
        let auth = client.auth.clone();

        let send: GcalSend<Value> = match self.method.to_ascii_uppercase().as_str() {
            "GET" => GcalSend::get(&auth, url),
            "DELETE" => GcalSend::delete(&auth, url),
            "POST" => GcalSend::post_json(&auth, url, &parse_body(self.body.as_deref())?)?,
            "PUT" => GcalSend::put_json(&auth, url, &parse_body(self.body.as_deref())?)?,
            "PATCH" => GcalSend::patch_json(&auth, url, &parse_body(self.body.as_deref())?)?,
            other => bail!("Unsupported method `{other}` (use get/post/put/patch/delete)"),
        };
        let out: GcalSendOutput<Value> = client.run(send)?;

        printer.out(RawJsonOutput {
            response: out.response,
        })
    }
}

/// Parses the optional raw body, an absent one meaning JSON null.
fn parse_body(body: Option<&str>) -> Result<Value> {
    serde_json::from_str(body.unwrap_or("null")).context("Parse request body JSON error")
}

/// The request URL: a path joined onto the Calendar base, a full URL
/// as-is when it is on Google's API origin and refused otherwise.
fn resolve_url(path: &str) -> Result<Url> {
    request_url(GCAL_API_BASE, path, "Google Calendar")
}

#[cfg(test)]
mod tests {
    use io_gcal::v3::client::GcalClientStd;
    use pimalaya_cli::printer::CapturePrinter;
    use serde_json::json;

    use super::*;
    use crate::{
        account::context::Account,
        script::{self, Script},
    };

    /// A path is joined onto the Calendar base, the body sent as given,
    /// and the answer prints raw under `response`.
    #[test]
    fn a_request_prints_the_raw_answer() {
        let answer = json!({"kind": "calendar#calendar", "id": "c1", "summary": "Work"});
        let (script, requests) = Script::new([script::response(
            200,
            "application/json",
            &answer.to_string(),
        )]);
        let client = GcalClient::new(
            GcalClientStd::new(script, "made-up"),
            "perso".into(),
            Account::default(),
        );

        let command = GcalRequestCommand {
            method: "post".into(),
            path: "/calendars".into(),
            body: Some(r#"{"summary":"Work"}"#.into()),
        };
        let mut printer = CapturePrinter::default();
        command.execute(&mut printer, client).unwrap();

        assert_eq!(printer.into_single().unwrap(), json!({"response": answer}));

        let requests = requests.lock().unwrap();
        let sent = String::from_utf8_lossy(&requests[0]);
        assert!(sent.starts_with("POST /calendar/v3/calendars "), "{sent}");
        assert!(sent.ends_with(r#"{"summary":"Work"}"#), "{sent}");
    }

    #[test]
    fn an_unknown_method_is_refused() {
        let (script, _) = Script::new([]);
        let client = GcalClient::new(
            GcalClientStd::new(script, "made-up"),
            "perso".into(),
            Account::default(),
        );
        let command = GcalRequestCommand {
            method: "head".into(),
            path: "colors".into(),
            body: None,
        };

        let mut printer = CapturePrinter::default();
        let err = command.execute(&mut printer, client).unwrap_err();
        assert!(err.to_string().contains("`HEAD`"), "{err}");
    }

    /// A full URL off Google's API origin is refused before anything is
    /// sent, so the token never reaches that host.
    #[test]
    fn a_url_off_the_api_origin_sends_nothing() {
        let (script, requests) = Script::new([]);
        let client = GcalClient::new(
            GcalClientStd::new(script, "made-up"),
            "perso".into(),
            Account::default(),
        );
        let command = GcalRequestCommand {
            method: "get".into(),
            path: "https://collector.example.org/calendar/v3/colors".into(),
            body: None,
        };

        let mut printer = CapturePrinter::default();
        let err = command.execute(&mut printer, client).unwrap_err();
        assert!(err.to_string().contains("collector.example.org"), "{err}");
        assert!(requests.lock().unwrap().is_empty());
    }
}
