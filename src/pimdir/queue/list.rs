//! # Pimdir queue list
//!
//! The `pimdir queue list` command, listing the creations and the intents
//! staged and the rows the store's owner parked in the account's
//! calendars.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{listing::Listing, printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::pimdir::backend::PimdirBackend;

/// List the items staged for creation, the replies and cancellations
/// staged, and the parked rows.
///
/// An item added on pimdir waits in the store's queue until the sync
/// engine applies it, and has no id until then, so `event list` cannot
/// show it; a reply or a cancellation waits there until the engine sends
/// it. This is where both show: the row id to cancel one by, when it was
/// queued, its kind, the link id `event add` reported or the event a reply
/// or a cancellation addresses, and its summary.
///
/// Staged updates and deletions need no such view, addressing events that
/// already exist. A row of any kind the sync engine parked, refusing to
/// apply it without an operator, is listed `parked` with why: cancel it,
/// or fix what parked it. One the engine sent without learning its outcome
/// is listed `unconfirmed` instead: it may have taken effect.
///
/// JSON output: `{"items": [{"row", "queuedAt", "producer", "collection",
/// "kind", "linkId", "itemId", "summary"}], "parked": [{"row", "queuedAt",
/// "producer", "collection", "kind", "attempts", "unconfirmed",
/// "error"}]}`.
#[derive(Debug, Parser)]
pub struct PimdirQueueListCommand {
    /// Calendar to list, every one of the account's when omitted.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR-ID")]
    pub calendar: Option<String>,
}

impl PimdirQueueListCommand {
    /// Lists the staged rows and the parked ones, and prints them.
    pub fn execute(self, printer: &mut impl Printer, backend: PimdirBackend) -> Result<()> {
        let calendar = self.calendar.as_deref();
        let items = backend.queued_rows(calendar)?;
        let parked = backend.parked_rows(calendar)?;

        printer.out(PimdirQueueListOutput {
            account: backend.name.clone(),
            calendar: self.calendar,
            items,
            parked,
        })
    }
}

/// One creation or intent waiting in the store's queue.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirQueuedRow {
    /// The queue row id, which `pimdir queue show` and `cancel` take. It
    /// names a pending action, not an item: a creation has no id until the
    /// sync engine applies it.
    #[serde(rename = "row")]
    pub id: i64,
    /// When the row was appended, stamped by the store's own clock.
    pub queued_at: String,
    /// The process that staged it.
    pub producer: String,
    /// The calendar the row is staged in.
    pub collection: String,
    /// The action kind: `add`, `calendar-reply` or `calendar-cancel`.
    pub kind: String,
    /// The link id `event add` or `item add` reported, derived from the
    /// item's `UID`, `null` for an intent naming a stored event.
    pub link_id: Option<String>,
    /// The event a reply or a cancellation addresses, `null` for a
    /// creation.
    pub item_id: Option<String>,
    /// The summary of the item, from the body a creation pins or the
    /// event an intent addresses, `null` when the store holds none.
    pub summary: Option<String>,
}

/// One row the sync engine parked: it will not apply it again without an
/// operator.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirParkedRow {
    /// The queue row id, which `pimdir queue cancel` takes.
    #[serde(rename = "row")]
    pub id: i64,
    /// When the row was appended, stamped by the store's own clock.
    pub queued_at: String,
    /// The process that staged it.
    pub producer: String,
    /// The calendar the row targets.
    pub collection: String,
    /// The action kind: `add`, `update`, `remove`, `calendar-reply`, ...
    pub kind: String,
    /// How many times the engine tried it before parking it.
    pub attempts: i64,
    /// Whether the row is unconfirmed: the engine sent its request and
    /// cannot tell whether it took effect, so it may have.
    pub unconfirmed: bool,
    /// Why it was parked, as the store records it: an unconfirmed row's
    /// starts with `unconfirmed: `.
    pub error: String,
}

/// The `pimdir queue list` output.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirQueueListOutput {
    /// The name of the account listed.
    #[serde(skip)]
    pub account: String,
    /// The calendar listed, `None` for every one of the account's.
    #[serde(skip)]
    pub calendar: Option<String>,
    /// The creations and the intents staged.
    pub items: Vec<PimdirQueuedRow>,
    /// The parked rows, any kind.
    pub parked: Vec<PimdirParkedRow>,
}

impl fmt::Display for PimdirQueueListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut listing = Listing::new(&self.account, Some("pimdir"));
        listing.head("Calendar", self.calendar.as_deref().unwrap_or("all"));
        listing.empty = Some("Nothing queued nor parked");

        for queued in &self.items {
            let what = queued.summary.as_deref().or(queued.item_id.as_deref());

            listing.items.push([
                queued.id.to_string(),
                String::from("queued"),
                sanitize::text(&queued.queued_at).into_owned(),
                sanitize::text(&queued.kind).into_owned(),
                sanitize::text(&queued.collection).into_owned(),
                sanitize::text(what.unwrap_or_default()).into_owned(),
            ]);
        }

        for parked in &self.parked {
            listing.items.push([
                parked.id.to_string(),
                String::from(match parked.unconfirmed {
                    true => "unconfirmed",
                    false => "parked",
                }),
                sanitize::text(&parked.queued_at).into_owned(),
                sanitize::text(&parked.kind).into_owned(),
                sanitize::text(&parked.collection).into_owned(),
                sanitize::text(&parked.error).into_owned(),
            ]);
        }

        if !listing.items.rows.is_empty() {
            let cancel = String::from("calendula pimdir queue cancel <ROW>");
            listing.header.push(("Cancel", cancel));
        }

        write!(f, "{listing}")
    }
}

#[cfg(test)]
mod tests {
    use super::{PimdirParkedRow, PimdirQueueListOutput, PimdirQueuedRow};

    /// The rows print one line each under the account and the calendar,
    /// the queued ones first, and the way to cancel one heads them; the
    /// JSON keys a row by `row`, its kind kebab-case, as Himalaya and
    /// cardamum do.
    #[test]
    fn the_rows_print_under_the_way_to_cancel_one() {
        let output = PimdirQueueListOutput {
            account: "perso".into(),
            calendar: None,
            items: vec![
                PimdirQueuedRow {
                    id: 3,
                    queued_at: "2026-11-15T09:00:00Z".into(),
                    producer: "calendula".into(),
                    collection: "cal".into(),
                    kind: "add".into(),
                    link_id: Some("standup@example.org".into()),
                    item_id: None,
                    summary: Some("Standup".into()),
                },
                PimdirQueuedRow {
                    id: 4,
                    queued_at: "2026-11-15T09:05:00Z".into(),
                    producer: "calendula".into(),
                    collection: "cal".into(),
                    kind: "calendar-reply".into(),
                    link_id: None,
                    item_id: Some("7".into()),
                    summary: None,
                },
            ],
            parked: vec![PimdirParkedRow {
                id: 2,
                queued_at: "2026-11-14T09:00:00Z".into(),
                producer: "calendula".into(),
                collection: "cal".into(),
                kind: "update".into(),
                attempts: 1,
                unconfirmed: false,
                error: "conflict".into(),
            }],
        };

        let expected = "\
Account:    perso (pimdir)
Calendar:   all
Cancel:     calendula pimdir queue cancel <ROW>

3  queued  2026-11-15T09:00:00Z  add             cal  Standup
4  queued  2026-11-15T09:05:00Z  calendar-reply  cal  7
2  parked  2026-11-14T09:00:00Z  update          cal  conflict
";
        assert_eq!(output.to_string(), expected);

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["items"][0]["row"], 3, "{json}");
        assert_eq!(json["items"][0]["linkId"], "standup@example.org");
        assert_eq!(json["items"][1]["itemId"], "7", "{json}");
        assert_eq!(json["parked"][0]["row"], 2, "{json}");
        assert_eq!(json["parked"][0]["collection"], "cal", "{json}");
    }

    /// An empty queue says so under its header.
    #[test]
    fn an_empty_queue_says_so() {
        let output = PimdirQueueListOutput {
            account: "perso".into(),
            calendar: Some("cal".into()),
            items: Vec::new(),
            parked: Vec::new(),
        };

        let expected = "\
Account:    perso (pimdir)
Calendar:   cal

Nothing queued nor parked
";
        assert_eq!(output.to_string(), expected);
    }
}
