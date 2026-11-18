//! # Pimdir queue show
//!
//! The `pimdir queue show` command, saying where one queue row stands.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use io_pimdir::{
    client::producer::{PimdirActionStatus, PimdirReceiptRefusal},
    codec::PimdirActionHandle,
};
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
/// id of the event it acted on (the one an add created). An applied row
/// lists the refusals standing on that event: a server that refused for
/// good the change the row led to, with its reason, held until the event
/// is edited or removed (pimdir STORAGE §15.4). The store keeps what
/// became of an applied row for seven days, and while a refusal stands on
/// its event, under the calendar it was queued on, or under none once that
/// calendar is gone; past that, and for a cancelled row, the state is
/// `unknown`, as for a row of another account.
///
/// JSON output: `{"row", "state", "collection", "kind", "attempts",
/// "error", "appliedAt", "seq", "refusals": [{"collection", "source",
/// "reason", "refusedAt"}]}`, as cardamum prints it.
#[derive(Debug, Parser)]
pub struct PimdirQueueShowCommand {
    /// The queue row: its id, as `pimdir queue list` prints it, or its
    /// handle, as a receipt prints it, refused once the store is a copy
    /// restored since (`reset`).
    #[arg(value_name = "ROW")]
    pub row: PimdirActionHandle,
}

impl PimdirQueueShowCommand {
    /// Looks the row up in the account's queue and receipts.
    pub fn execute(self, printer: &mut impl Printer, backend: PimdirBackend) -> Result<()> {
        let status = backend.queue_row(&self.row)?;
        printer.out(PimdirQueueShowOutput::new(self.row.id, status))
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
    /// or a cancellation sent); `seq` is the event it acted on, and
    /// `refusals` the servers that refused the change it led to.
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
    /// The id of the event an applied row acted on (the one an add
    /// created), as `event read` takes it.
    pub seq: Option<i64>,
    /// The refusals standing on that event, empty but for an applied row.
    pub refusals: Vec<PimdirQueueRefusal>,
}

/// A refusal standing on the event an applied row acted on (pimdir
/// STORAGE §15.4): the server of `source` refused for good the change the
/// row led to, and the sync engine holds it until the event changes again.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirQueueRefusal {
    /// The calendar filing the event, the target of a move.
    pub collection: String,
    /// The source whose server refused the change.
    pub source: String,
    /// The text the server gave.
    pub reason: String,
    /// When the refusal was recorded, RFC 3339.
    pub refused_at: String,
}

impl From<PimdirReceiptRefusal> for PimdirQueueRefusal {
    fn from(refusal: PimdirReceiptRefusal) -> Self {
        Self {
            collection: refusal.collection,
            source: refusal.source,
            reason: refusal.reason,
            refused_at: refusal.refused_at,
        }
    }
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
            refusals: Vec::new(),
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
                refusals,
            } => Self {
                state: PimdirQueueState::Applied,
                collection,
                applied_at: Some(applied_at),
                seq,
                refusals: refusals.into_iter().map(Into::into).collect(),
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
                    write!(f, ", on event {seq}")?;
                }
                writeln!(f)?;
                for refusal in &self.refusals {
                    writeln!(
                        f,
                        "Refused by {} in {} at {}: {}",
                        sanitize::text(&refusal.source),
                        sanitize::text(&refusal.collection),
                        sanitize::text(&refusal.refused_at),
                        sanitize::text(&refusal.reason),
                    )?;
                }
                Ok(())
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
                refusals: Vec::new(),
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
                "refusals": [],
            })
        );
    }

    /// An applied row whose change a server refused for good shows applied,
    /// then a line per refusal with its source and reason (pimdir STORAGE
    /// §15.4), and the JSON lists them.
    #[test]
    fn an_applied_row_prints_the_refusals_on_its_event() {
        let row = PimdirQueueShowOutput::new(
            12,
            PimdirActionStatus::Applied {
                applied_at: "2026-11-15T20:00:00.000Z".into(),
                collection: Some("work".into()),
                seq: Some(42),
                refusals: vec![PimdirReceiptRefusal {
                    collection: "work".into(),
                    source: "caldav".into(),
                    reason: "403 Forbidden".into(),
                    refused_at: "2026-11-15T20:05:00.000Z".into(),
                }],
            },
        );

        let json = serde_json::to_value(&row).unwrap();
        assert_eq!(json["state"], "applied");
        assert_eq!(
            json["refusals"],
            serde_json::json!([{
                "collection": "work",
                "source": "caldav",
                "reason": "403 Forbidden",
                "refusedAt": "2026-11-15T20:05:00.000Z",
            }])
        );
        assert_eq!(
            row.to_string(),
            "Row 12 was applied in work, on event 42\n\
             Refused by caldav in work at 2026-11-15T20:05:00.000Z: 403 Forbidden\n"
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
                refusals: Vec::new(),
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
                "refusals": [],
            })
        );
    }
}
