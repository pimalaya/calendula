//! # Pimdir queue show
//!
//! The `pimdir queue show` command, saying where one queue row stands.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use io_pimdir::client::producer::PimdirActionStatus;
use pimalaya_cli::{printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::pimdir::backend::PimdirBackend;

/// Show where one queue row stands, by the row `pimdir queue list`,
/// `pimdir reply` or `pimdir cancel` printed.
///
/// A row is `pending` until the sync engine applies it, `parked` when the
/// engine gave up on it (with why), `unconfirmed` when the engine sent its
/// request and cannot tell whether it took effect (a reply or a
/// cancellation that may have gone out: it waits for the engine to confirm
/// it, or for a cancellation of the row), and `applied` once done, with the
/// id of the event an add created. The store keeps what became of an
/// applied row for seven days, under the calendar it was queued on, or
/// under none once that calendar is gone; past that, and for a cancelled
/// row, the state is `unknown`, as for a row of another account.
///
/// JSON output: `{"row", "state", "collection", "kind", "attempts",
/// "error", "appliedAt", "seq"}`, as Himalaya prints it.
#[derive(Debug, Parser)]
pub struct PimdirQueueShowCommand {
    /// The queue row id.
    #[arg(value_name = "ROW")]
    pub id: i64,
}

impl PimdirQueueShowCommand {
    /// Looks the row up in the account's queue and receipts.
    pub fn execute(self, printer: &mut impl Printer, backend: PimdirBackend) -> Result<()> {
        let status = backend.queue_row(self.id)?;
        printer.out(PimdirQueueShowOutput::new(self.id, status))
    }
}

/// Where a queue row stands.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum PimdirQueueState {
    /// Waiting for the sync engine.
    Pending,
    /// Given up on by the sync engine; `error` says why.
    Parked,
    /// Sent by the sync engine, which cannot tell whether it took effect:
    /// it may have, and waits for the engine to confirm it or for a
    /// cancellation; `error` holds what the engine recorded.
    Unconfirmed,
    /// Applied or performed by the sync engine (an event written, a reply
    /// or a cancellation sent); `seq` is the event an add created.
    Applied,
    /// Neither queued nor known applied: cancelled, or applied long ago.
    Unknown,
    /// The store is not the one the row was queued in, a copy restored
    /// since, where the id may name another row: nothing is read of it.
    Reset,
}

/// The `pimdir queue show` output.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirQueueShowOutput {
    /// The queue row id.
    #[serde(rename = "row")]
    pub queue_id: i64,
    /// Where it stands.
    pub state: PimdirQueueState,
    /// The calendar it is anchored on: `null` when unknown, and for an
    /// applied row whose calendar is gone since.
    pub collection: Option<String>,
    /// The action kind (`add`, `calendar-reply`, ...), while queued.
    pub kind: Option<String>,
    /// Apply attempts so far, while queued.
    pub attempts: Option<i64>,
    /// Why the row was parked, or what the sync engine recorded of an
    /// unconfirmed one.
    pub error: Option<String>,
    /// When the row was applied, RFC 3339.
    pub applied_at: Option<String>,
    /// The id of the event an applied add created, in `collection`.
    pub seq: Option<i64>,
}

impl PimdirQueueShowOutput {
    /// Projects io-pimdir's status of row `queue_id`.
    pub fn new(queue_id: i64, status: PimdirActionStatus) -> Self {
        let row = Self {
            queue_id,
            state: PimdirQueueState::Unknown,
            collection: None,
            kind: None,
            attempts: None,
            error: None,
            applied_at: None,
            seq: None,
        };

        match status {
            PimdirActionStatus::Pending {
                collection,
                kind,
                attempts,
            } => Self {
                state: PimdirQueueState::Pending,
                collection: Some(collection),
                kind: Some(kind),
                attempts: Some(attempts),
                ..row
            },
            PimdirActionStatus::Parked {
                collection,
                kind,
                attempts,
                error,
            } => Self {
                state: PimdirQueueState::Parked,
                collection: Some(collection),
                kind: Some(kind),
                attempts: Some(attempts),
                error: Some(error),
                ..row
            },
            PimdirActionStatus::Unconfirmed {
                collection,
                kind,
                attempts,
                detail,
            } => Self {
                state: PimdirQueueState::Unconfirmed,
                collection: Some(collection),
                kind: Some(kind),
                attempts: Some(attempts),
                error: Some(detail),
                ..row
            },
            PimdirActionStatus::Applied {
                applied_at,
                collection,
                seq,
            } => Self {
                state: PimdirQueueState::Applied,
                collection,
                applied_at: Some(applied_at),
                seq,
                ..row
            },
            PimdirActionStatus::Unknown => row,
            PimdirActionStatus::Reset => Self {
                state: PimdirQueueState::Reset,
                ..row
            },
        }
    }
}

impl fmt::Display for PimdirQueueShowOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = self.queue_id;
        let collection = sanitize::text(self.collection.as_deref().unwrap_or_default());
        let kind = sanitize::text(self.kind.as_deref().unwrap_or_default());

        match self.state {
            PimdirQueueState::Pending => {
                writeln!(f, "Row {id} ({kind} in {collection}) is pending")
            }
            PimdirQueueState::Parked => {
                let error = sanitize::text(self.error.as_deref().unwrap_or_default());
                writeln!(f, "Row {id} ({kind} in {collection}) is parked: {error}")
            }
            PimdirQueueState::Unconfirmed => {
                let detail = sanitize::text(self.error.as_deref().unwrap_or_default());
                writeln!(
                    f,
                    "Row {id} ({kind} in {collection}) is unconfirmed, sent and maybe \
                     performed: {detail}"
                )
            }
            PimdirQueueState::Applied => {
                match self.collection {
                    Some(_) => write!(f, "Row {id} was applied in {collection}")?,
                    None => write!(f, "Row {id} was applied in a calendar deleted since")?,
                }
                if let Some(seq) = self.seq {
                    write!(f, ", creating event {seq}")?;
                }
                writeln!(f)
            }
            PimdirQueueState::Unknown => writeln!(
                f,
                "Row {id} is neither queued nor known applied: cancelled, or applied long ago"
            ),
            PimdirQueueState::Reset => writeln!(
                f,
                "Row {id} was queued in another store: a copy restored since, where this id \
                 may name another row"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_applied_add_prints_its_seq() {
        let row = PimdirQueueShowOutput::new(
            12,
            PimdirActionStatus::Applied {
                applied_at: "2026-10-04T20:00:00Z".into(),
                collection: Some("personal".into()),
                seq: Some(42),
            },
        );
        let json = serde_json::to_value(&row).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "row": 12,
                "state": "applied",
                "collection": "personal",
                "kind": null,
                "attempts": null,
                "error": null,
                "appliedAt": "2026-10-04T20:00:00Z",
                "seq": 42,
            })
        );
    }

    /// An intent its performer sent and could not confirm is no failure
    /// and no success (pimdir STORAGE §15.2): it reads `unconfirmed`, with
    /// what the performer recorded, never `parked`.
    #[test]
    fn an_unconfirmed_row_says_it_may_have_taken_effect() {
        let row = PimdirQueueShowOutput::new(
            9,
            PimdirActionStatus::Unconfirmed {
                collection: "work".into(),
                kind: "calendar-reply".into(),
                attempts: 1,
                detail: "connection lost after the request".into(),
            },
        );

        let json = serde_json::to_value(&row).unwrap();
        assert_eq!(json["state"], "unconfirmed");
        assert_eq!(json["kind"], "calendar-reply");
        assert_eq!(json["error"], "connection lost after the request");

        let line = row.to_string();
        assert!(line.contains("unconfirmed"), "{line}");
        assert!(line.contains("maybe performed"), "{line}");
        assert!(line.contains("connection lost after the request"), "{line}");
    }

    /// A row applied on a calendar deleted since keeps its receipt and
    /// names no collection (pimdir STORAGE §15.4).
    #[test]
    fn an_applied_row_of_a_deleted_calendar_names_no_collection() {
        let row = PimdirQueueShowOutput::new(
            4,
            PimdirActionStatus::Applied {
                applied_at: "2026-11-13T09:00:00Z".into(),
                collection: None,
                seq: None,
            },
        );

        let json = serde_json::to_value(&row).unwrap();
        assert_eq!(json["state"], "applied");
        assert_eq!(json["collection"], serde_json::Value::Null);
        assert!(row.to_string().contains("deleted since"), "{row}");
    }

    #[test]
    fn an_unknown_row_prints_its_id_and_state() {
        let json = serde_json::to_value(PimdirQueueShowOutput::new(7, PimdirActionStatus::Unknown))
            .unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "row": 7,
                "state": "unknown",
                "collection": null,
                "kind": null,
                "attempts": null,
                "error": null,
                "appliedAt": null,
                "seq": null,
            })
        );
    }
}
