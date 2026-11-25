//! # Multiget report command
//!
//! Runs the RFC 4791 `calendar-multiget` REPORT and renders the fetched
//! items.

use anyhow::Result;
use clap::Parser;
use io_webdav::rfc4918::typed::WebdavUnreadableEntry;
use pimalaya_cli::printer::Printer;

use crate::{
    caldav::{
        backend::unreadable_row,
        client::CaldavClient,
        report::entries::{CaldavItemEntriesOutput, EntryRow},
    },
    error::{ErrorCode, ItemError, partial_when},
};

/// Fetch the named items in one round-trip (RFC 4791 §7.9).
///
/// The `calendar-multiget` REPORT: one request brings back the body and
/// ETag of every id given.
///
/// JSON output: `{"items": [{"id", "etag", "contents", "encoding"}],
/// "truncated", "errors": [{"id", "code", "error", "sources"}]}`: the error
/// of each item the server answered without its data, `event-not-found`
/// for an id it does not hold, `[]` when none, the command then exiting 3.
#[derive(Debug, Parser)]
pub struct CaldavReportMultigetCommand {
    /// Identifier of the calendar to query.
    #[arg(value_name = "CALENDAR")]
    pub calendar_id: String,
    /// Item resource ids to fetch.
    #[arg(value_name = "ITEM-ID", required = true)]
    pub item_ids: Vec<String>,
}

impl CaldavReportMultigetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CaldavClient) -> Result<()> {
        let ids: Vec<&str> = self.item_ids.iter().map(String::as_str).collect();
        let ok = client.multiget_items(&self.calendar_id, &ids)?;

        let output = CaldavItemEntriesOutput {
            rows: ok.items.into_iter().map(EntryRow::from).collect(),
            truncated: false,
            errors: ok.unreadable.into_iter().map(multiget_row).collect(),
        };

        printer.out(&output)?;
        partial_when(&output.errors)
    }
}

/// The entry of an item the multiget named and the server answered
/// without its data: a row answered 404 or 410 is `event-not-found`, as an
/// item request is, any other reads as a listing row.
fn multiget_row(row: WebdavUnreadableEntry) -> ItemError {
    match row.status {
        Some(status @ (404 | 410)) => ItemError {
            id: Some(row.id),
            code: Some(ErrorCode::EventNotFound),
            error: format!("The server holds no such item, status {status}"),
            sources: row.failures.iter().map(ToString::to_string).collect(),
        },
        _ => unreadable_row(row),
    }
}
