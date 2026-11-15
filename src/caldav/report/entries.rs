//! # Item entries
//!
//! The item list the `query` and `multiget` REPORT commands print.

use std::fmt;

use ical::{
    component::{vevent::VEVENT, vjournal::VJOURNAL, vtodo::VTODO},
    prop::summary::SUMMARY,
    tree::cst::IcalCst,
};
use io_webdav::rfc4791::item::CaldavItemEntry;
use pimalaya_cli::{columns::Columns, error::ItemErrors, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{error::ItemError, shared::item::ItemContents};

/// Items returned by a `query` or `multiget` REPORT.
///
/// The text lists ids, ETags and summaries, then the errors; the raw
/// iCalendar body rides in `contents` for `--json`.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CaldavItemEntriesOutput {
    /// The items the REPORT returned.
    #[serde(rename = "items")]
    pub rows: Vec<EntryRow>,
    /// Whether the server truncated the result, more items matching than
    /// it returned.
    pub truncated: bool,
    /// The error of each item the server answered without its data, in
    /// the order met.
    pub errors: Vec<ItemError>,
}

/// One returned item: its id, ETag and raw iCalendar body.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EntryRow {
    /// Item resource id, its href last path segment.
    pub id: String,
    /// The server ETag, when it sent one.
    pub etag: Option<String>,
    /// The raw iCalendar, as text or as base64.
    #[serde(flatten)]
    pub contents: ItemContents,
    /// The first `SUMMARY` of the item, for the text output only.
    #[serde(skip)]
    pub summary: Option<String>,
}

impl From<CaldavItemEntry> for EntryRow {
    fn from(entry: CaldavItemEntry) -> Self {
        Self {
            summary: summary(&entry.data),
            contents: ItemContents::of(&entry.data),
            id: entry.id,
            etag: entry.etag,
        }
    }
}

impl fmt::Display for CaldavItemEntriesOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.rows.is_empty() {
            let mut columns = Columns::new(["ID", "ETAG", "SUMMARY"]);

            for entry in &self.rows {
                let text = |value: &Option<String>| match value {
                    Some(value) => sanitize::text(value).into_owned(),
                    None => String::new(),
                };

                columns.push([
                    sanitize::text(&entry.id).into_owned(),
                    text(&entry.etag),
                    text(&entry.summary),
                ]);
            }

            write!(f, "{columns}")?;
        }

        if self.truncated {
            if !self.rows.is_empty() {
                writeln!(f)?;
            }

            writeln!(f, "The server truncated the result: more items match.")?;
        }

        let mut errors = ItemErrors::default();

        for error in &self.errors {
            errors.push(error);
        }

        // NOTE: the blank line parts the errors from what printed above,
        // so none leads an output holding the errors alone.
        match self.rows.is_empty() && !self.truncated {
            true => write!(f, "{errors}"),
            false => errors.write_after(f),
        }
    }
}

/// The first `SUMMARY` of an item's event, to-do or journal entry.
fn summary(data: &[u8]) -> Option<String> {
    let cst = IcalCst::parse(data).ok()?;
    let mut components = cst
        .components::<VEVENT>()
        .chain(cst.components::<VTODO>())
        .chain(cst.components::<VJOURNAL>());

    components.find_map(|component| {
        let summary = component.prop::<SUMMARY>()?;
        Some(summary.0.into_owned())
    })
}

#[cfg(test)]
mod tests {
    use anyhow::{Result, anyhow};
    use clap::Parser;
    use io_webdav::{client::WebdavClientStd, rfc4918::WebdavAuth};
    use pimalaya_cli::{error::partial_outcome_of, printer::CapturePrinter};
    use serde_json::{Value, json};
    use url::Url;

    use super::{CaldavItemEntriesOutput, EntryRow, summary};
    use crate::{
        caldav::{
            client::CaldavClient,
            report::{multiget::CaldavReportMultigetCommand, query::CaldavReportQueryCommand},
        },
        error::ItemError,
        script::{self, Script},
        shared::item::ItemContents,
    };

    #[test]
    fn reads_the_first_summary() {
        let item = b"BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:e1\r\n\
                     SUMMARY:Stand\\, up\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        assert_eq!(summary(item).as_deref(), Some("Stand, up"));
        assert_eq!(summary(b"BEGIN:VCALENDAR\r\nEND:VCALENDAR\r\n"), None);
    }

    /// A client over a CalDAV server answering `body` with a 207, its home
    /// set already discovered.
    fn answering(body: &str) -> CaldavClient {
        let (script, _) = Script::new([script::response(207, "application/xml", body)]);
        let base = Url::parse("https://dav.example.org/").unwrap();
        let home = base.join("/dav/").unwrap();
        let inner =
            WebdavClientStd::from_parts(script, WebdavAuth::None, base, None, Some(home), None);
        CaldavClient::new(inner)
    }

    /// The JSON a command printed and the exit status it ends with: 0, or
    /// the status of its partial outcome.
    fn run(execute: impl FnOnce(&mut CapturePrinter) -> Result<()>) -> (Value, i32) {
        let mut printer = CapturePrinter::default();
        let status = match execute(&mut printer) {
            Ok(()) => 0,
            Err(err) => partial_outcome_of(&err)
                .expect("a partial outcome")
                .status(),
        };
        (printer.into_single().unwrap(), status)
    }

    /// The `(id, code, error, sources)` of each entry of `json`'s errors.
    fn entries(json: &Value) -> Vec<Value> {
        let errors = json["errors"].as_array().unwrap();
        let fields = |entry: &Value| {
            let keys = ["id", "code", "error", "sources"];
            Value::Array(keys.iter().map(|key| entry[key].clone()).collect())
        };
        errors.iter().map(fields).collect()
    }

    const E1: &str = r#"
  <d:response>
    <d:href>/dav/cal/e1.ics</d:href>
    <d:propstat>
      <d:prop>
        <d:getetag>"e1"</d:getetag>
        <c:calendar-data>BEGIN:VCALENDAR&#13;
VERSION:2.0&#13;
BEGIN:VEVENT&#13;
UID:e1&#13;
SUMMARY:Standup&#13;
END:VEVENT&#13;
END:VCALENDAR&#13;
</c:calendar-data>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>"#;

    const E2_REFUSED: &str = r#"
  <d:response>
    <d:href>/dav/cal/e2.ics</d:href>
    <d:status>HTTP/1.1 403 Forbidden</d:status>
  </d:response>"#;

    /// A multistatus holding `responses`.
    fn multistatus(responses: &[&str]) -> String {
        let open = r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">"#;
        format!("{open}{}\n</d:multistatus>", responses.concat())
    }

    /// Each row a query answered without the item's data is an entry, as
    /// `item list` codes it, the item read printing as an item row does.
    #[test]
    fn a_query_names_the_rows_it_could_not_read() {
        let body = multistatus(&[E1, E2_REFUSED]);

        let cmd = CaldavReportQueryCommand::try_parse_from(["query", "cal"]).unwrap();
        let (json, status) = run(|printer| cmd.execute(printer, answering(&body)));

        assert_eq!(json["items"][0]["id"], "e1.ics");
        assert_eq!(json["items"][0]["etag"], "e1");
        assert_eq!(json["items"][0]["encoding"], "utf-8");
        assert!(
            json["items"][0]["contents"]
                .as_str()
                .unwrap()
                .ends_with("END:VCALENDAR\r\n")
        );
        assert_eq!(json["items"].as_array().unwrap().len(), 1);
        assert_eq!(json["truncated"], false);
        assert_eq!(
            entries(&json),
            [json!([
                "e2.ics",
                "permission-denied",
                "The server refused the item with status 403",
                []
            ])]
        );
        assert_eq!(status, 3);
    }

    /// An id the server does not hold is `item-not-found`.
    #[test]
    fn a_multiget_names_the_items_it_could_not_read() {
        let e9 = r#"
  <d:response>
    <d:href>/dav/cal/e9.ics</d:href>
    <d:status>HTTP/1.1 404 Not Found</d:status>
  </d:response>"#;
        let body = multistatus(&[E1, e9]);

        let argv = ["multiget", "cal", "e1.ics", "e9.ics"];
        let cmd = CaldavReportMultigetCommand::try_parse_from(argv).unwrap();
        let (json, status) = run(|printer| cmd.execute(printer, answering(&body)));

        assert_eq!(json["items"][0]["id"], "e1.ics");
        assert_eq!(
            entries(&json),
            [json!([
                "e9.ics",
                "item-not-found",
                "The server holds no such item, status 404",
                []
            ])]
        );
        assert_eq!(status, 3);
    }

    /// The errors print last, after the table, the truncation and one
    /// blank line.
    #[test]
    fn the_errors_print_last_after_one_blank_line() {
        let output = CaldavItemEntriesOutput {
            rows: vec![EntryRow {
                id: String::from("e1.ics"),
                etag: Some(String::from("e1")),
                contents: ItemContents::default(),
                summary: Some(String::from("Standup")),
            }],
            truncated: true,
            errors: vec![ItemError::new(Some("e2.ics"), &anyhow!("Refused"))],
        };

        let expected = "\
ID      ETAG  SUMMARY
e1.ics  e1    Standup

The server truncated the result: more items match.

Error:  e2.ics  -  Refused
";
        assert_eq!(output.to_string(), expected);
    }
}
