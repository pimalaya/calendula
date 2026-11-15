//! # GET command
//!
//! The `calendula caldav get` command, fetching the raw iCalendar bytes of
//! one item with a WebDAV `GET`.

use core::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    caldav::client::{CaldavClient, item_refusal},
    shared::item::{ItemContents, read::printable},
};

/// GET an item: its raw iCalendar bytes plus the server ETag.
///
/// JSON output: `{"id", "etag", "contents", "encoding"}`, `contents` the
/// iCalendar text (`encoding` `utf-8`) or the base64 of bytes that are not
/// UTF-8 (`encoding` `base64`).
#[derive(Debug, Parser)]
pub struct CaldavGetCommand {
    /// Identifier of the parent calendar.
    #[arg(value_name = "CALENDAR")]
    pub calendar_id: String,
    /// Item resource id (its href last path segment).
    #[arg(value_name = "ITEM-ID")]
    pub item_id: String,
}

impl CaldavGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CaldavClient) -> Result<()> {
        let body = client
            .read_item(&self.calendar_id, &self.item_id)
            .map_err(item_refusal)?;

        printer.out(CaldavGetOutput {
            id: self.item_id,
            etag: body.etag,
            contents: ItemContents::of(&body.data),
            bytes: body.data,
        })
    }
}

/// A fetched item: its raw iCalendar bytes and the server ETag.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CaldavGetOutput {
    /// Item resource id, its href last path segment.
    pub id: String,
    /// The server ETag, when it sent one.
    pub etag: Option<String>,
    /// The raw iCalendar, as text or as base64.
    #[serde(flatten)]
    pub contents: ItemContents,
    /// The bytes as stored, for the text view.
    #[serde(skip)]
    pub bytes: Vec<u8>,
}

impl fmt::Display for CaldavGetOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", printable(&self.bytes).trim_end())
    }
}
