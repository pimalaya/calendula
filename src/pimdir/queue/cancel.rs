//! # Pimdir queue cancel
//!
//! The `pimdir queue cancel` command, retracting one queued or parked
//! action before its owner applies it.

use std::fmt;

use anyhow::{Result, bail};
use clap::Parser;
use io_pimdir::{client::producer::PimdirActionStatus, codec::PimdirActionHandle};
use pimalaya_cli::{printer::Printer, prompt, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::pimdir::backend::PimdirBackend;

/// Cancel one queued or parked action, by the row id `queue list` prints.
///
/// This is the only way back for a queued creation, an event that does not
/// exist yet being one no `event delete` reaches, and for a queued reply or
/// cancellation before the sync engine sends it. Only a row of the
/// account's queue is taken.
///
/// An unconfirmed row, a reply or a cancellation the sync engine sent
/// without learning its outcome, is cancelled the same way, which undoes
/// nothing: it may have gone out, and both the question and the answer say
/// so.
///
/// JSON output: `{"row", "handle", "kind", "unconfirmed"}`, as Himalaya
/// prints it.
///
/// Cancelling is the store owner's write, so this takes that role for the
/// length of the call and fails while a sync holds it. A store another
/// version of the sync engine wrote is refused rather than reshaped.
#[derive(Debug, Parser)]
pub struct PimdirQueueCancelCommand {
    /// The queued action's row: its id, as `pimdir queue list` prints
    /// it, or its handle, as a receipt prints it, refused `store-reset`
    /// once the store is a copy restored since.
    #[arg(value_name = "ROW")]
    pub row: PimdirActionHandle,
    /// Do not ask for confirmation.
    #[arg(long, short)]
    pub yes: bool,
}

impl PimdirQueueCancelCommand {
    /// Retracts the queued action the row id names.
    pub fn execute(self, printer: &mut impl Printer, backend: PimdirBackend) -> Result<()> {
        let (named, unconfirmed) = match backend.queue_row(&self.row)? {
            PimdirActionStatus::Pending { kind, .. } | PimdirActionStatus::Parked { kind, .. } => {
                (kind, false)
            }
            PimdirActionStatus::Unconfirmed { kind, .. } => (kind, true),
            _ => (String::from("action"), false),
        };
        let question = match unconfirmed {
            true => format!(
                "The {} queued as row {} was sent and may have taken effect, which \
                 cancelling does not undo. Cancel it?",
                sanitize::text(&named),
                self.row
            ),
            false => format!(
                "Cancel the {} queued as row {}?",
                sanitize::text(&named),
                self.row
            ),
        };
        if !self.yes && !prompt::bool(question, false)? {
            bail!("Cancellation aborted");
        }

        let handle = backend.named(&self.row)?;
        let kind = backend.cancel_queued(&self.row)?;
        printer.out(PimdirQueueCancelOutput {
            row: handle.id,
            handle: handle.to_string(),
            kind,
            unconfirmed,
        })
    }
}

/// The `pimdir queue cancel` output.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirQueueCancelOutput {
    /// The queue row id cancelled.
    pub row: i64,
    /// The row's handle, its id under the store's identity (pimdir STORAGE
    /// §4.5): the handle given, or the bare id given named under this
    /// store's identity.
    pub handle: String,
    /// The action kind it held: `add`, `update`, `calendar-reply`, ...
    pub kind: String,
    /// Whether the row was unconfirmed, its request sent and its outcome
    /// unknown: the action may have taken effect all the same.
    pub unconfirmed: bool,
}

impl fmt::Display for PimdirQueueCancelOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = sanitize::text(&self.kind);
        let id = &self.handle;
        match self.unconfirmed {
            true => writeln!(
                f,
                "Unconfirmed {kind} cancelled (row {id}); it was sent and may have taken effect"
            ),
            false => writeln!(f, "Queued {kind} cancelled (row {id})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cancelling an unconfirmed row drops it from the queue and undoes
    /// nothing its performer sent (pimdir STORAGE §15.2), so the output
    /// never reads as a plain cancellation.
    #[test]
    fn cancelling_an_unconfirmed_row_says_it_may_have_taken_effect() {
        let cancelled = PimdirQueueCancelOutput {
            row: 9,
            handle: "e2646df031cd876334410c4df14ebd86.1.9".into(),
            kind: "calendar-reply".into(),
            unconfirmed: true,
        };

        let json = serde_json::to_value(&cancelled).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "row": 9,
                "handle": "e2646df031cd876334410c4df14ebd86.1.9",
                "kind": "calendar-reply",
                "unconfirmed": true,
            })
        );
        let line = cancelled.to_string();
        assert!(line.contains("may have taken effect"), "{line}");
    }
}
