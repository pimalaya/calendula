//! # Sync report command
//!
//! Runs the RFC 6578 `sync-collection` REPORT and renders the changed
//! and vanished hrefs.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use io_webdav::rfc6578::sync_collection::{
    WebdavSyncChange, WebdavSyncCollectionOptions, WebdavSyncDelta,
};
use pimalaya_cli::{columns::Columns, error::ItemErrors, printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    caldav::{backend::unreadable_row, client::CaldavClient},
    error::{ItemError, partial_when},
};

/// Report the changes since a sync token (RFC 6578).
///
/// Omit `--sync-token` for an initial sync, then feed the returned
/// token back to the next call. Only ETags are fetched, so pair with
/// `report multiget` to pull the changed items.
///
/// A member the server answered with an error is neither changed nor
/// vanished: it is an `errors` entry, and the command exits 3.
///
/// JSON output: `{"changed": [{"href", "etag"}], "vanished": [...],
/// "syncToken", "truncated", "errors"}`.
#[derive(Debug, Parser)]
pub struct CaldavReportSyncCommand {
    /// Identifier of the calendar to sync.
    #[arg(value_name = "CALENDAR")]
    pub calendar_id: String,
    /// Sync token from a previous sync; omit for an initial sync.
    #[arg(long, value_name = "TOKEN")]
    pub sync_token: Option<String>,
    /// Enumerate with a Depth 1 PROPFIND instead of the REPORT.
    ///
    /// For a server implementing no `sync-collection`: it lists every
    /// member and returns no token, so the result is a full snapshot to
    /// diff.
    #[arg(long)]
    pub fallback: bool,
}

impl CaldavReportSyncCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CaldavClient) -> Result<()> {
        let opts = WebdavSyncCollectionOptions {
            fallback: self.fallback,
        };
        let delta = client.sync_items(&self.calendar_id, self.sync_token.as_deref(), opts)?;
        let output = CaldavReportSyncOutput::new(delta);

        printer.out(&output)?;
        partial_when(&output.errors)
    }
}

/// Changes a `sync-collection` REPORT reported for one calendar.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CaldavReportSyncOutput {
    /// The resources created or updated since the given token.
    pub changed: Vec<ChangeRow>,
    /// The hrefs of the resources that vanished since it.
    pub vanished: Vec<String>,
    /// The token to feed to the next sync, when the server sent one.
    pub sync_token: Option<String>,
    /// Whether the server cut the listing short (RFC 6578 section 3.6).
    pub truncated: bool,
    /// The error of each member the server answered with an error status,
    /// its state unknown, in the order met.
    pub errors: Vec<ItemError>,
}

/// One created or updated resource: its href and ETag.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChangeRow {
    /// The changed resource's href.
    pub href: String,
    /// Its ETag, when the server sent one.
    pub etag: Option<String>,
}

impl From<WebdavSyncChange> for ChangeRow {
    fn from(change: WebdavSyncChange) -> Self {
        Self {
            href: change.href,
            etag: change.etag,
        }
    }
}

impl CaldavReportSyncOutput {
    fn new(delta: WebdavSyncDelta) -> Self {
        Self {
            changed: delta.changed.into_iter().map(ChangeRow::from).collect(),
            vanished: delta.vanished,
            sync_token: delta.sync_token,
            truncated: delta.truncated,
            errors: delta.unreadable.into_iter().map(unreadable_row).collect(),
        }
    }
}

impl fmt::Display for CaldavReportSyncOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.changed.is_empty() || !self.vanished.is_empty() {
            let mut columns = Columns::new(["STATUS", "ETAG", "HREF"]);

            for change in &self.changed {
                let etag = change.etag.as_deref().map(sanitize::text);

                columns.push([
                    String::from("changed"),
                    etag.map(|etag| etag.into_owned()).unwrap_or_default(),
                    sanitize::text(&change.href).into_owned(),
                ]);
            }

            for href in &self.vanished {
                columns.push([
                    String::from("vanished"),
                    String::new(),
                    sanitize::text(href).into_owned(),
                ]);
            }

            writeln!(f, "{columns}")?;
        }

        let token = self.sync_token.as_deref().map(sanitize::text);
        let token = token.unwrap_or("(none)".into());
        writeln!(f, "sync-token: {token}")?;
        if self.truncated {
            writeln!(
                f,
                "truncated: run sync again from this token to drain the rest"
            )?;
        }

        let mut errors = ItemErrors::default();

        for error in &self.errors {
            errors.push(error);
        }

        errors.write_after(f)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fmt,
        io::{Cursor, Read, Result as IoResult, Write},
    };

    use anyhow::Result;
    use io_webdav::{client::WebdavClientStd, rfc4918::WebdavAuth};
    use pimalaya_cli::{error::PartialOutcome, output::Bytes, printer::Printer};
    use serde::Serialize;
    use serde_json::{Value, json};
    use url::Url;

    use super::CaldavReportSyncCommand;
    use crate::caldav::client::CaldavClient;

    /// A server answering one canned HTTP response, whatever it is sent.
    struct Canned(Cursor<Vec<u8>>);

    impl Read for Canned {
        fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
            self.0.read(buf)
        }
    }

    impl Write for Canned {
        fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
            Ok(buf.len())
        }

        fn flush(&mut self) -> IoResult<()> {
            Ok(())
        }
    }

    /// A printer keeping what it was handed, as JSON.
    #[derive(Default)]
    struct Capture(Vec<Value>);

    impl Printer for Capture {
        fn out<T: fmt::Display + Serialize>(&mut self, data: T) -> Result<()> {
            self.0.push(serde_json::to_value(data)?);
            Ok(())
        }

        fn event<T: fmt::Display + Serialize>(&mut self, _event: T) -> Result<()> {
            Ok(())
        }

        fn out_bytes(&mut self, _bytes: Bytes) -> Result<()> {
            Ok(())
        }

        fn is_json(&self) -> bool {
            true
        }
    }

    /// A `sync-collection` answer: `e1.ics` changed, `e2.ics` refused 403,
    /// `e3.ics` removed.
    const DELTA: &str = r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:">
  <d:response><d:href>/calendars/me/cal/e1.ics</d:href><d:propstat><d:prop><d:getetag>"1"</d:getetag></d:prop><d:status>HTTP/1.1 200 OK</d:status></d:propstat></d:response>
  <d:response><d:href>/calendars/me/cal/e2.ics</d:href><d:status>HTTP/1.1 403 Forbidden</d:status></d:response>
  <d:response><d:href>/calendars/me/cal/e3.ics</d:href><d:status>HTTP/1.1 404 Not Found</d:status></d:response>
  <d:sync-token>https://dav.example.org/sync/2</d:sync-token>
</d:multistatus>"#;

    /// A member the server answered with an error is neither changed nor
    /// vanished: it is an `errors` entry coded from its status, and the
    /// command exits 3, so a diff of the listing infers no removal.
    #[test]
    fn a_member_the_server_refused_is_an_error_and_exits_3() {
        let response = format!(
            "HTTP/1.1 207 Multi-Status\r\nContent-Type: application/xml\r\n\
             Content-Length: {}\r\n\r\n{DELTA}",
            DELTA.len()
        );
        let home = Url::parse("https://dav.example.org/calendars/me/").unwrap();
        let stream = Canned(Cursor::new(response.into_bytes()));
        let mut inner = WebdavClientStd::new(stream, WebdavAuth::None, home.clone());
        inner.calendar_home_set = Some(home);

        let command = CaldavReportSyncCommand {
            calendar_id: String::from("cal"),
            sync_token: None,
            fallback: false,
        };
        let mut printer = Capture::default();
        let err = command
            .execute(&mut printer, CaldavClient::new(inner))
            .unwrap_err();
        assert!(err.downcast_ref::<PartialOutcome>().is_some(), "{err:#}");

        let output = printer.0.remove(0);
        assert_eq!(output["changed"][0]["href"], "/calendars/me/cal/e1.ics");
        assert_eq!(output["vanished"], json!(["/calendars/me/cal/e3.ics"]));
        assert_eq!(
            output["errors"],
            json!([{
                "id": "e2.ics",
                "code": "permission-denied",
                "error": "The server refused the item with status 403",
                "sources": [],
            }]),
            "{output}"
        );
    }
}
