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
/// JSON output: `{"items": [<row>], "parked": [<row>]}`, each row, pending
/// or parked, `{"row", "kind", "collection", "linkId", "itemId",
/// "producer", "queuedAt", "attempts", "unconfirmed", "error",
/// "summary"}`, as every Pimalaya program prints a queue row.
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

/// One row of the store's queue, pending or parked, in the one shape
/// every Pimalaya program prints it (pimalaya-cli's output contract, "A
/// queue row keeps one shape"), every key printed.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirQueueRow {
    /// The queue row id, which `pimdir queue show` and `cancel` take. It
    /// names a queued action, not an item: a creation has no id until the
    /// sync engine applies it.
    pub row: i64,
    /// The row's action as the store holds it, never re-spelled: `add`,
    /// `update`, `remove`, `calendar-reply`, `calendar-cancel`, ...
    pub kind: String,
    /// The calendar the row is queued on.
    pub collection: String,
    /// The link id of the item the row creates, the one `event add` or
    /// `item add` reported, `null` when the row creates none.
    pub link_id: Option<String>,
    /// The id of the stored item the row addresses (an update, a removal,
    /// a reply or a cancellation of a stored event), as `item list` prints
    /// it, `null` when it addresses none, a creation among them.
    pub item_id: Option<String>,
    /// The process that staged the row.
    pub producer: String,
    /// When the row was appended, stamped by the store's own clock.
    pub queued_at: String,
    /// How many times the sync engine tried the row.
    pub attempts: i64,
    /// Whether the row's outcome is unknown: the engine sent its request
    /// and cannot tell whether it took effect, so it may have.
    pub unconfirmed: bool,
    /// Why the row was parked, as the store records it, an unconfirmed
    /// row's beginning `unconfirmed: `; `null` while the row is pending.
    pub error: Option<String>,
    /// The summary of the body the row pins, else of the item it
    /// addresses, `null` when there is none to read.
    pub summary: Option<String>,
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
    pub items: Vec<PimdirQueueRow>,
    /// The parked rows, any kind.
    pub parked: Vec<PimdirQueueRow>,
}

impl fmt::Display for PimdirQueueListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut listing = Listing::new(&self.account, Some("pimdir"));
        listing.head("Calendar", self.calendar.as_deref().unwrap_or("all"));
        listing.empty = Some("Nothing queued nor parked");

        for queued in &self.items {
            let what = queued.summary.as_deref().or(queued.item_id.as_deref());
            listing.items.push(cells(queued, "queued", what));
        }

        for parked in &self.parked {
            let state = match parked.unconfirmed {
                true => "unconfirmed",
                false => "parked",
            };
            listing
                .items
                .push(cells(parked, state, parked.error.as_deref()));
        }

        if !listing.items.rows.is_empty() {
            let cancel = String::from("calendula pimdir queue cancel <ROW>");
            listing.header.push(("Cancel", cancel));
        }

        write!(f, "{listing}")
    }
}

/// The text cells of `row`: its id, its `state`, when it was queued, its
/// kind, its calendar, then `last`.
fn cells(row: &PimdirQueueRow, state: &str, last: Option<&str>) -> [String; 6] {
    [
        row.row.to_string(),
        String::from(state),
        sanitize::text(&row.queued_at).into_owned(),
        sanitize::text(&row.kind).into_owned(),
        sanitize::text(&row.collection).into_owned(),
        sanitize::text(last.unwrap_or_default()).into_owned(),
    ]
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{PimdirQueueListOutput, PimdirQueueRow};

    /// A row of `kind` in the calendar `cal`, pending.
    fn row(row: i64, kind: &str) -> PimdirQueueRow {
        PimdirQueueRow {
            row,
            kind: kind.into(),
            collection: "cal".into(),
            link_id: None,
            item_id: None,
            producer: "calendula".into(),
            queued_at: "2026-11-15T09:00:00Z".into(),
            attempts: 0,
            unconfirmed: false,
            error: None,
            summary: None,
        }
    }

    /// The rows print one line each under the account and the calendar,
    /// the queued ones first, and the way to cancel one heads them; the
    /// JSON prints a pending and a parked row in one shape, every key
    /// present, as Himalaya and cardamum do.
    #[test]
    fn the_rows_print_under_the_way_to_cancel_one() {
        let output = PimdirQueueListOutput {
            account: "perso".into(),
            calendar: None,
            items: vec![
                PimdirQueueRow {
                    link_id: Some("standup@example.org".into()),
                    summary: Some("Standup".into()),
                    ..row(3, "add")
                },
                PimdirQueueRow {
                    item_id: Some("7".into()),
                    queued_at: "2026-11-15T09:05:00Z".into(),
                    ..row(4, "calendar-reply")
                },
            ],
            parked: vec![PimdirQueueRow {
                item_id: Some("5".into()),
                queued_at: "2026-11-14T09:00:00Z".into(),
                attempts: 1,
                error: Some("conflict".into()),
                summary: Some("Review".into()),
                ..row(2, "update")
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
        assert_eq!(
            json["items"][0],
            json!({
                "row": 3,
                "kind": "add",
                "collection": "cal",
                "linkId": "standup@example.org",
                "itemId": null,
                "producer": "calendula",
                "queuedAt": "2026-11-15T09:00:00Z",
                "attempts": 0,
                "unconfirmed": false,
                "error": null,
                "summary": "Standup",
            })
        );
        assert_eq!(
            json["parked"][0],
            json!({
                "row": 2,
                "kind": "update",
                "collection": "cal",
                "linkId": null,
                "itemId": "5",
                "producer": "calendula",
                "queuedAt": "2026-11-14T09:00:00Z",
                "attempts": 1,
                "unconfirmed": false,
                "error": "conflict",
                "summary": "Review",
            })
        );
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
