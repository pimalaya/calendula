//! # Raw report command
//!
//! Sends an arbitrary XML REPORT body to a calendar and renders the
//! multistatus it answers.

use std::{
    fmt, fs,
    io::{Read, stdin},
    path::PathBuf,
};

use anyhow::{Context, Result, anyhow, bail};
use clap::Parser;
use io_webdav::{
    rfc4791::calendar::CALENDAR_DATA,
    rfc4918::{
        GETETAG, WebdavMultistatus, WebdavResponseEntry, collection_path, report::WebdavReport,
    },
};
use pimalaya_cli::{columns::Columns, printer::Printer, sanitize, session::check_stdin};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    caldav::client::{CaldavClient, check_calendar_id},
    error::ErrorCode,
    mcp::paths_closed,
};

/// Run a REPORT with an arbitrary XML body against a calendar.
///
/// The escape hatch for the `calendar-query`, `calendar-multiget`,
/// `free-busy-query` and `sync-collection` variants the typed
/// subcommands do not cover. The XML body comes from a file, inline, or
/// `-` for stdin, and the parsed multistatus is printed.
///
/// JSON output: `{"responses": [{"href", "status", "etag",
/// "dataBytes"}], "syncToken"}`.
#[derive(Debug, Parser)]
pub struct CaldavReportRawCommand {
    /// Identifier of the calendar to run the REPORT against.
    #[arg(value_name = "CALENDAR")]
    pub calendar_id: String,
    /// XML REPORT body: a file path, raw XML, or `-` for stdin.
    #[arg(value_name = "XML")]
    pub xml: String,
    /// `Depth` header (0 or 1).
    #[arg(long, default_value_t = 1)]
    pub depth: u8,
}

impl CaldavReportRawCommand {
    /// Runs the REPORT on the calendar's path, built as io-webdav builds it
    /// for its own requests: an id addressing another collection is
    /// refused first, a relative one percent-encoded. It goes through the
    /// client, sent again on a fresh connection when the server closed the
    /// one it was sent on.
    pub fn execute(self, printer: &mut impl Printer, mut client: CaldavClient) -> Result<()> {
        check_calendar_id(&self.calendar_id)?;
        let body = read_xml(&self.xml)?;

        let home = client
            .calendar_home_set
            .as_ref()
            .ok_or_else(|| anyhow!("Calendar home-set is not resolved"))?;
        let path = collection_path(home.path(), &self.calendar_id);

        let coroutine = WebdavReport::<WebdavMultistatus>::new(
            &client.base_url,
            client.auth(),
            &client.user_agent,
            &path,
            self.depth,
            body,
        );
        let multistatus = client.run(coroutine)?;

        printer.out(CaldavReportRawOutput {
            sync_token: multistatus.sync_token,
            responses: multistatus.responses.iter().map(EntryRow::from).collect(),
        })
    }
}

/// Reads the XML body from stdin (`-`), a file, or the value itself.
///
/// An existing path is read from disk, otherwise a value starting with
/// `<` is taken as inline XML.
fn read_xml(source: &str) -> Result<Vec<u8>> {
    if source == "-" {
        check_stdin(ErrorCode::StdinTaken)?;
        let mut buf = Vec::new();
        stdin()
            .read_to_end(&mut buf)
            .context("Read REPORT body from stdin error")?;
        return Ok(buf);
    }

    let path = PathBuf::from(source);

    if path.is_file() && !paths_closed() {
        return fs::read(&path)
            .with_context(|| format!("Read REPORT body from `{}` error", path.display()));
    }

    if source.trim_start().starts_with('<') {
        return Ok(source.as_bytes().to_vec());
    }

    bail!("Source `{source}` is neither a readable file nor XML");
}

/// Multistatus a raw REPORT answered.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CaldavReportRawOutput {
    /// One entry per multistatus response.
    pub responses: Vec<EntryRow>,
    /// The `sync-token` the multistatus carried, when it carried one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_token: Option<String>,
}

/// One multistatus response: href, status, ETag and body size.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EntryRow {
    /// The response href.
    pub href: String,
    /// Its status code, when the response carried one.
    pub status: Option<u16>,
    /// Its `getetag` property, unquoted.
    pub etag: Option<String>,
    /// Size of its `calendar-data` property, in bytes: the whole item, its
    /// final line break included.
    pub data_bytes: usize,
}

impl From<&WebdavResponseEntry> for EntryRow {
    fn from(entry: &WebdavResponseEntry) -> Self {
        Self {
            href: entry.href.clone(),
            status: entry.status,
            etag: entry
                .text(GETETAG)
                .map(|raw| raw.trim_matches('"').to_string()),
            // NOTE: `text` trims the final CRLF; the property keeps it whole.
            data_bytes: entry
                .prop(CALENDAR_DATA)
                .map(|prop| prop.text.len())
                .unwrap_or(0),
        }
    }
}

impl fmt::Display for CaldavReportRawOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.responses.is_empty() {
            let mut columns = Columns::new(["STATUS", "DATA-BYTES", "ETAG", "HREF"]);

            for entry in &self.responses {
                let status = entry.status.map(|s| s.to_string()).unwrap_or_default();
                let etag = entry.etag.as_deref().map(sanitize::text);

                columns.push([
                    status,
                    entry.data_bytes.to_string(),
                    etag.map(|etag| etag.into_owned()).unwrap_or_default(),
                    sanitize::text(&entry.href).into_owned(),
                ]);
            }

            write!(f, "{columns}")?;
        }

        if let Some(token) = &self.sync_token {
            if !self.responses.is_empty() {
                writeln!(f)?;
            }

            writeln!(f, "sync-token: {}", sanitize::text(token))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::ErrorKind;

    use io_webdav::{
        client::{WebdavClientStd, WebdavStream},
        rfc4918::WebdavAuth,
    };
    use pimalaya_cli::{
        error::{code_of, with_code},
        printer::CapturePrinter,
    };
    use url::Url;

    use super::CaldavReportRawCommand;
    use crate::{
        caldav::client::CaldavClient,
        error::{ErrorCode, code},
        script::{self, Requests, Script},
    };

    /// The item the server answers, each line ending in a CRLF.
    const ITEM: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\n\
                        BEGIN:VEVENT\r\nUID:e1\r\nDTSTAMP:20260101T000000Z\r\n\
                        DTSTART:20260814T090000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

    /// A multistatus answering [`ITEM`], each CR escaped as iCloud and
    /// Fastmail escape it.
    const MULTISTATUS: &str = r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:response>
    <d:href>/dav/My%20Cal/e1.ics</d:href>
    <d:propstat>
      <d:prop>
        <d:getetag>"e1"</d:getetag>
        <c:calendar-data>BEGIN:VCALENDAR&#13;
VERSION:2.0&#13;
PRODID:-//x//y//EN&#13;
BEGIN:VEVENT&#13;
UID:e1&#13;
DTSTAMP:20260101T000000Z&#13;
DTSTART:20260814T090000Z&#13;
END:VEVENT&#13;
END:VCALENDAR&#13;
</c:calendar-data>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
</d:multistatus>"#;

    /// A client whose home set is `/dav/`, over a server answering
    /// [`MULTISTATUS`], and the requests it receives.
    fn client() -> (CaldavClient, Requests) {
        let (script, requests) = answering();
        (over(script), requests)
    }

    /// A server answering [`MULTISTATUS`], and the requests it receives.
    fn answering() -> (Script, Requests) {
        Script::new([script::response(207, "application/xml", MULTISTATUS)])
    }

    /// A client whose home set is `/dav/`, over `script`.
    fn over(script: Script) -> CaldavClient {
        let base = Url::parse("https://dav.example.org/").unwrap();
        let home = base.join("/dav/").unwrap();
        let inner =
            WebdavClientStd::from_parts(script, WebdavAuth::None, base, None, Some(home), None);
        CaldavClient::new(inner)
    }

    /// The command running a REPORT on `calendar_id`.
    fn report(calendar_id: &str) -> CaldavReportRawCommand {
        CaldavReportRawCommand {
            calendar_id: calendar_id.to_owned(),
            xml: String::from("<x/>"),
            depth: 1,
        }
    }

    /// The size printed is the whole item's, its final CRLF included, and
    /// the path the one io-webdav builds, percent-encoded.
    #[test]
    fn a_raw_report_counts_the_whole_item_on_an_encoded_path() {
        let (client, requests) = client();
        let mut printer = CapturePrinter::default();
        report("My Cal").execute(&mut printer, client).unwrap();

        let output = printer.into_single().unwrap();
        assert_eq!(output["responses"][0]["dataBytes"], ITEM.len(), "{output}");

        let requests = requests.lock().unwrap();
        let request = String::from_utf8_lossy(&requests[0]);
        assert!(
            request.starts_with("REPORT /dav/My%20Cal/ HTTP/1.1\r\n"),
            "{request}"
        );
    }

    /// A server that closed the connection after the previous request (an
    /// idle keep-alive timing out) gets the REPORT again on a fresh one, as
    /// every io-webdav request does, and its answer is printed.
    #[test]
    fn a_raw_report_is_sent_again_on_a_fresh_connection() {
        let (closed, sent) = Script::new([]);
        let mut client = over(closed);
        let (fresh, resent) = answering();
        let mut fresh = Some(fresh);
        client.set_reconnect(Box::new(move || {
            let fresh = fresh.take().ok_or(ErrorKind::NotConnected)?;
            Ok(Box::new(fresh) as Box<dyn WebdavStream>)
        }));

        let mut printer = CapturePrinter::default();
        report("My Cal").execute(&mut printer, client).unwrap();

        let output = printer.into_single().unwrap();
        assert_eq!(output["responses"][0]["dataBytes"], ITEM.len(), "{output}");

        let sent = sent.lock().unwrap();
        let resent = resent.lock().unwrap();
        assert_eq!(*resent, *sent);
        let request = String::from_utf8_lossy(&resent[0]);
        assert!(
            request.starts_with("REPORT /dav/My%20Cal/ HTTP/1.1\r\n"),
            "{request}"
        );
    }

    /// A refused REPORT carries the code of any CalDAV refusal, its message
    /// the server's status.
    #[test]
    fn a_refused_raw_report_is_coded() {
        let response = script::response(403, "text/plain", "Forbidden");
        let (script, _) = Script::new([response]);
        let mut printer = CapturePrinter::default();
        let err = report("cal")
            .execute(&mut printer, over(script))
            .map_err(|err| with_code(err, code))
            .unwrap_err();

        assert_eq!(code_of(&err), Some(ErrorCode::PermissionDenied), "{err:#}");
        assert!(err.to_string().contains("HTTP 403"), "{err:#}");
    }

    /// An id addressing the home set, its parent or the server root is
    /// refused `calendar-not-found`, nothing sent.
    #[test]
    fn a_raw_report_refuses_an_id_out_of_its_home() {
        for id in ["..", "", "/", ".", "a/../..", "%2e%2e", "a//b"] {
            let (client, requests) = client();
            let mut printer = CapturePrinter::default();
            let err = report(id).execute(&mut printer, client).unwrap_err();

            assert_eq!(code_of(&err), Some(ErrorCode::CalendarNotFound), "{id:?}");
            assert_eq!(err.to_string(), format!("Calendar `{id}` not found"));
            assert!(requests.lock().unwrap().is_empty(), "{id:?}");
        }
    }
}
