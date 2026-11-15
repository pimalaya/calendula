//! # PUT command
//!
//! The `calendula caldav put` command, creating or replacing the raw
//! iCalendar bytes of an item with a WebDAV `PUT`.

use core::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    caldav::client::{CaldavClient, item_refusal},
    shared::ical::read_source,
};

/// PUT an item: create or replace its raw iCalendar bytes.
///
/// A plain PUT creates or overwrites unconditionally. `--if-none-match
/// '*'` makes it a create-only PUT, failing when the resource exists, and
/// `--if-match <ETAG>` gates the replace on the resource being unchanged
/// (RFC 9110). The iCalendar goes as it is given, unchecked.
///
/// JSON output: `{"id", "etag"}`.
#[derive(Debug, Parser)]
pub struct CaldavPutCommand {
    /// Identifier of the parent calendar.
    #[arg(value_name = "CALENDAR")]
    pub calendar_id: String,
    /// Item resource id (its href last path segment).
    #[arg(value_name = "ITEM-ID")]
    pub item_id: String,
    /// iCalendar written to the item resource: a path to a file, raw
    /// iCalendar contents, or `-` for stdin.
    #[arg(value_name = "ICAL")]
    pub ical: String,
    /// Gate the replace on the resource being unchanged (If-Match).
    #[arg(long, value_name = "ETAG", conflicts_with = "if_none_match")]
    pub if_match: Option<String>,
    /// Create-only PUT: fail if the resource already exists (pass `*`).
    #[arg(long, value_name = "VALUE", conflicts_with = "if_match")]
    pub if_none_match: Option<String>,
}

impl CaldavPutCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CaldavClient) -> Result<()> {
        let contents = read_source(&self.ical)?;

        let (id, etag) = if self.if_none_match.is_some() {
            let created = client.create_item(&self.calendar_id, &self.item_id, contents)?;
            (created.id, created.etag)
        } else {
            let updated = client
                .update_item(
                    &self.calendar_id,
                    &self.item_id,
                    contents,
                    self.if_match.as_deref(),
                )
                .map_err(item_refusal)?;
            (updated.id, updated.etag)
        };

        printer.out(CaldavPutOutput { id, etag })
    }
}

/// The item resource `caldav put` wrote.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CaldavPutOutput {
    /// The item resource id.
    pub id: String,
    /// The ETag the server answered, `null` when it sent none.
    pub etag: Option<String>,
}

impl fmt::Display for CaldavPutOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let etag = self.etag.as_deref().unwrap_or("(none)");
        let etag = sanitize::text(etag);
        let id = sanitize::text(&self.id);
        writeln!(f, "Item `{id}` successfully put (etag {etag})")
    }
}
