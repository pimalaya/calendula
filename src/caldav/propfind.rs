//! # PROPFIND command
//!
//! The `calendula caldav propfind` command, enumerating the calendar
//! collections under the resolved home-set, or the item resources of one,
//! with a WebDAV `PROPFIND`.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use io_webdav::rfc4791::{calendar::CaldavCalendar, item::CaldavItemRef};
use pimalaya_cli::{
    columns::Columns,
    fields::Fields,
    printer::Printer,
    records::{Record, Records},
    sanitize,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{caldav::client::CaldavClient, shared::calendar::list::body};

/// PROPFIND the home-set or a calendar.
///
/// Without a calendar, lists the calendar collections with what only
/// CalDAV carries: the collection change tag, the sync token an
/// incremental sync starts from, and the component kinds the server
/// accepts. With one, enumerates its item resources with ids and ETags but
/// no bodies, through the `calendar-query` REPORT asking `getetag` alone,
/// as io-webdav enumerates a collection.
///
/// JSON output: `{"calendars": [{"id", "displayName", "description",
/// "color", "components", "ctag", "syncToken"}]}` (no arg) or `{"items":
/// [{"id", "etag"}], "truncated"}` (with arg).
#[derive(Debug, Parser)]
pub struct CaldavPropfindCommand {
    /// Calendar to enumerate; omit to list the calendar collections under
    /// the home-set.
    #[arg(value_name = "CALENDAR")]
    pub calendar_id: Option<String>,
}

impl CaldavPropfindCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CaldavClient) -> Result<()> {
        match self.calendar_id {
            None => {
                let rows = client
                    .list_calendars()?
                    .into_iter()
                    .map(CalendarRow::from)
                    .collect();

                printer.out(CaldavPropfindOutput::Calendars(
                    CaldavPropfindCalendarsOutput { rows },
                ))
            }
            Some(id) => {
                let enumerated = client.enum_items(&id, "")?;

                printer.out(CaldavPropfindOutput::Items(CaldavPropfindItemsOutput {
                    rows: enumerated.refs.into_iter().map(ItemRefRow::from).collect(),
                    truncated: enumerated.truncated,
                }))
            }
        }
    }
}

/// What a PROPFIND prints, which is what it was pointed at.
///
/// Untagged, so each shape serializes exactly as it would on its own: the
/// argument already says which of the two came back.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum CaldavPropfindOutput {
    /// The calendar collections under the home-set.
    Calendars(CaldavPropfindCalendarsOutput),
    /// The item resources of one calendar.
    Items(CaldavPropfindItemsOutput),
}

impl fmt::Display for CaldavPropfindOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Calendars(out) => out.fmt(f),
            Self::Items(out) => out.fmt(f),
        }
    }
}

/// The calendar collections a home-set PROPFIND lists.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CaldavPropfindCalendarsOutput {
    /// One entry per calendar collection.
    #[serde(rename = "calendars")]
    pub rows: Vec<CalendarRow>,
}

/// One CalDAV calendar with the properties only CalDAV carries.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CalendarRow {
    /// The last path segment of the collection URL.
    pub id: String,
    /// The DAV display name.
    pub display_name: Option<String>,
    /// The free-form description (RFC 4791 6.2.1).
    pub description: Option<String>,
    /// The display color (RFC 7986 5.9).
    pub color: Option<String>,
    /// The component kinds the collection accepts (RFC 4791 5.2.3).
    ///
    /// Empty means the server declares no restriction.
    pub components: Vec<String>,
    /// The collection change tag, bumped on every change.
    pub ctag: Option<String>,
    /// The RFC 6578 sync token an incremental sync starts from, to feed
    /// to `report sync`.
    pub sync_token: Option<String>,
}

impl From<CaldavCalendar> for CalendarRow {
    fn from(calendar: CaldavCalendar) -> Self {
        Self {
            id: calendar.id,
            display_name: calendar.display_name,
            description: calendar.description,
            color: calendar.color,
            components: calendar.components.into_iter().collect(),
            ctag: calendar.ctag,
            sync_token: calendar.sync_token,
        }
    }
}

impl fmt::Display for CaldavPropfindCalendarsOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut records = Records::default();

        for calendar in &self.rows {
            let mut fields = Fields::default();
            let name = calendar.display_name.as_deref().unwrap_or_default();
            fields.push("Name", sanitize::text(name));

            let optional = [
                ("Color", calendar.color.as_deref()),
                ("Ctag", calendar.ctag.as_deref()),
                ("Sync token", calendar.sync_token.as_deref()),
            ];

            for (label, value) in optional {
                if let Some(value) = value {
                    fields.push(label, sanitize::text(value));
                }
            }

            if !calendar.components.is_empty() {
                let components = calendar.components.join(", ");
                fields.push("Components", sanitize::text(&components));
            }

            let description = calendar.description.as_deref().unwrap_or_default();

            records.push(Record {
                title: sanitize::text(&calendar.id).into_owned(),
                state: Vec::new(),
                fields,
                body: body(description),
            });
        }

        write!(f, "{records}")
    }
}

/// The item references a calendar PROPFIND enumerates.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CaldavPropfindItemsOutput {
    /// The item resources the calendar holds.
    #[serde(rename = "items")]
    pub rows: Vec<ItemRefRow>,
    /// Whether the server cut the listing short with a 507 row.
    ///
    /// Such a listing (RFC 6578 §3.6) is a part of the calendar rather
    /// than the whole of it.
    pub truncated: bool,
}

/// One item resource: its id and ETag, never its body.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemRefRow {
    /// Item resource id, its href last path segment.
    pub id: String,
    /// The server ETag, when it sent one.
    pub etag: Option<String>,
}

impl From<CaldavItemRef> for ItemRefRow {
    fn from(item: CaldavItemRef) -> Self {
        Self {
            id: item.id,
            etag: item.etag,
        }
    }
}

impl fmt::Display for CaldavPropfindItemsOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.rows.is_empty() {
            let mut columns = Columns::new(["ID", "ETAG"]);

            for item in &self.rows {
                let etag = item.etag.as_deref().unwrap_or_default();
                columns.push([
                    sanitize::text(&item.id).into_owned(),
                    sanitize::text(etag).into_owned(),
                ]);
            }

            write!(f, "{columns}")?;
        }

        if self.truncated {
            if !self.rows.is_empty() {
                writeln!(f)?;
            }

            writeln!(f, "The server truncated the listing: more items exist.")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use io_webdav::{client::WebdavClientStd, rfc4918::WebdavAuth};
    use pimalaya_cli::printer::CapturePrinter;
    use serde_json::{Value, json};
    use url::Url;

    use super::CaldavPropfindCommand;
    use crate::{
        caldav::client::CaldavClient,
        script::{self, Requests, Script},
    };

    /// A home holding the calendar `cal`, its CalDAV properties set.
    const CALENDARS: &str = r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav" xmlns:cs="http://calendarserver.org/ns/">
  <d:response>
    <d:href>/dav/cal/</d:href>
    <d:propstat>
      <d:prop>
        <d:resourcetype><d:collection/><c:calendar/></d:resourcetype>
        <d:displayname>Cal</d:displayname>
        <cs:getctag>c1</cs:getctag>
        <d:sync-token>data:,1</d:sync-token>
        <c:supported-calendar-component-set><c:comp name="VEVENT"/></c:supported-calendar-component-set>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
</d:multistatus>"#;

    /// A calendar holding `e1.ics`.
    const ITEMS: &str = r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:">
  <d:response>
    <d:href>/dav/cal/e1.ics</d:href>
    <d:propstat>
      <d:prop><d:getetag>"e1"</d:getetag></d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
</d:multistatus>"#;

    /// A client whose home set is `/dav/`, over a server answering `body`.
    fn client(body: &str) -> (CaldavClient, Requests) {
        let (script, requests) = Script::new([script::response(207, "application/xml", body)]);
        let base = Url::parse("https://dav.example.org/").unwrap();
        let home = base.join("/dav/").unwrap();
        let inner =
            WebdavClientStd::from_parts(script, WebdavAuth::None, base, None, Some(home), None);
        (CaldavClient::new(inner), requests)
    }

    /// What `propfind` prints for `calendar_id`.
    fn propfind(calendar_id: Option<&str>, body: &str) -> (Value, String) {
        let (client, requests) = client(body);
        let cmd = CaldavPropfindCommand {
            calendar_id: calendar_id.map(str::to_owned),
        };
        let mut printer = CapturePrinter::default();
        cmd.execute(&mut printer, client).unwrap();

        let requests = requests.lock().unwrap();
        let request = String::from_utf8_lossy(&requests[0]).into_owned();
        (printer.into_single().unwrap(), request)
    }

    /// With no calendar, the listing keeps the shape `caldav list`
    /// printed, key for key.
    #[test]
    fn a_home_propfind_keeps_the_former_listing_shape() {
        let (json, request) = propfind(None, CALENDARS);

        assert!(request.starts_with("PROPFIND /dav/ "), "{request}");
        assert_eq!(
            json,
            json!({"calendars": [{
                "id": "cal",
                "displayName": "Cal",
                "description": null,
                "color": null,
                "components": ["VEVENT"],
                "ctag": "c1",
                "syncToken": "data:,1",
            }]})
        );
    }

    /// With a calendar, its items by id and entity tag, no body.
    #[test]
    fn a_calendar_propfind_enumerates_its_items() {
        let (json, request) = propfind(Some("cal"), ITEMS);

        assert!(request.starts_with("REPORT /dav/cal/ "), "{request}");
        assert_eq!(
            json,
            json!({"items": [{"id": "e1.ics", "etag": "e1"}], "truncated": false})
        );
    }
}
