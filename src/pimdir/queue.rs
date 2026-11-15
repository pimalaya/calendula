//! # Pimdir queue
//!
//! The `pimdir queue` commands, over the writes and intents staged in the
//! store's queue for its owner to apply or perform, as Himalaya's: list
//! the rows staged in the account's calendars, show where one stands,
//! cancel one.

pub mod cancel;
pub mod list;
pub mod show;

use anyhow::{Result, bail};
use clap::Subcommand;
use io_pimdir::{
    client::{PimdirError, PimdirStore, producer::PimdirActionStatus},
    codec::PimdirAction,
    intent::{PimdirIntentItem, PimdirInvitation},
    summary::{PimdirSummary, calendar::derive},
};
use pimalaya_cli::{error::CodedError, printer::Printer};

use crate::{
    error::ErrorCode,
    pimdir::{
        backend::PimdirBackend,
        client::store_error,
        queue::{
            cancel::PimdirQueueCancelCommand,
            list::{PimdirParkedRow, PimdirQueueListCommand, PimdirQueuedRow},
            show::PimdirQueueShowCommand,
        },
    },
};

/// Read and retract the writes staged in the store's queue.
///
/// A pimdir store is a replica its sync engine owns, so a write, a reply
/// or a cancellation is appended to the store's queue and applied or sent
/// on the engine's next run. A staged update or deletion shows in the
/// ordinary listing straight away; a staged creation has no id until the
/// engine applies it, and an intent no item at all, which is what these
/// commands are for.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum PimdirQueueCommand {
    #[command(alias = "ls")]
    List(PimdirQueueListCommand),
    Show(PimdirQueueShowCommand),
    Cancel(PimdirQueueCancelCommand),
}

impl PimdirQueueCommand {
    pub fn execute(self, printer: &mut impl Printer, backend: PimdirBackend) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.execute(printer, backend),
            Self::Show(cmd) => cmd.execute(printer, backend),
            Self::Cancel(cmd) => cmd.execute(printer, backend),
        }
    }
}

impl PimdirBackend {
    /// The calendars a queue listing reads: `calendar` alone, refused
    /// `calendar-not-found` unless the account holds it, else every one of
    /// the account's.
    fn queue_calendars(&self, calendar: Option<&str>) -> Result<Vec<String>> {
        match calendar {
            Some(id) => {
                self.known_collection(id)?;
                Ok(vec![id.to_owned()])
            }
            None => Ok(self
                .calendar_collections()?
                .into_iter()
                .map(|collection| collection.id)
                .collect()),
        }
    }

    /// The creations and the intents waiting in the queue of `calendar`,
    /// or of every calendar of the account, in append order.
    ///
    /// An update or a removal is left out: it addresses an item that
    /// already exists, which the listings show it on.
    pub(crate) fn queued_rows(&self, calendar: Option<&str>) -> Result<Vec<PimdirQueuedRow>> {
        let mut rows = Vec::new();

        for collection in self.queue_calendars(calendar)? {
            let pending = self
                .client
                .reader
                .pending_actions(&collection)
                .map_err(|err| store_error(err, "Read the pimdir queue"))?;

            for queued in pending {
                let (link_id, item_id, summary) = match &queued.action {
                    PimdirAction::Add {
                        link_id, object, ..
                    } => {
                        let body = object
                            .as_ref()
                            .and_then(|hash| self.client.blobs.get(hash).ok().flatten());
                        let summary = body.and_then(|body| title(derive(&body).summary?));
                        (link_id.as_ref().map(|id| id.0.clone()), None, summary)
                    }
                    PimdirAction::Unknown { kind, payload, .. } => {
                        match PimdirInvitation::from_payload(kind, payload).map(|it| it.item) {
                            Ok(PimdirIntentItem::Seq(seq)) => {
                                let id = seq.to_string();
                                let stored = self.item(&collection, &id).ok();
                                let summary = stored.and_then(|item| title(item.summary?));
                                (None, Some(id), summary)
                            }
                            Ok(PimdirIntentItem::Link(link_id)) => (Some(link_id.0), None, None),
                            Err(_) => (None, None, None),
                        }
                    }
                    _ => continue,
                };

                rows.push(PimdirQueuedRow {
                    id: queued.id,
                    queued_at: queued.created_at,
                    producer: queued.producer,
                    collection: queued.collection,
                    kind: queued.action.kind().to_owned(),
                    link_id,
                    item_id,
                    summary,
                });
            }
        }

        rows.sort_by_key(|row| row.id);
        Ok(rows)
    }

    /// The rows the store's owner parked in `calendar`, or in every
    /// calendar of the account, in append order.
    pub(crate) fn parked_rows(&self, calendar: Option<&str>) -> Result<Vec<PimdirParkedRow>> {
        let calendars = self.queue_calendars(calendar)?;

        Ok(self
            .client
            .reader
            .parked_actions()
            .map_err(|err| store_error(err, "Read the pimdir queue"))?
            .into_iter()
            .filter(|parked| calendars.contains(&parked.collection))
            .map(|parked| PimdirParkedRow {
                unconfirmed: parked.unconfirmed().is_some(),
                id: parked.id,
                queued_at: parked.created_at,
                producer: parked.producer,
                collection: parked.collection,
                kind: parked.action,
                attempts: parked.attempts,
                error: parked.error,
            })
            .collect())
    }

    /// Where the queue row `id` stands, `Unknown` for a row of a calendar
    /// another account holds.
    pub(crate) fn queue_row(&self, id: i64) -> Result<PimdirActionStatus> {
        let status = self
            .client
            .reader
            .action_status(id)
            .map_err(|err| store_error(err, format!("Read queue row {id}")))?;

        let collection = match &status {
            PimdirActionStatus::Pending { collection, .. }
            | PimdirActionStatus::Parked { collection, .. }
            | PimdirActionStatus::Unconfirmed { collection, .. }
            | PimdirActionStatus::Applied {
                collection: Some(collection),
                ..
            } => collection,
            PimdirActionStatus::Applied {
                collection: None, ..
            }
            | PimdirActionStatus::Unknown
            | PimdirActionStatus::Reset => return Ok(status),
        };

        let ours = self
            .calendar_collections()?
            .iter()
            .any(|ours| &ours.id == collection);

        Ok(match ours {
            true => status,
            false => PimdirActionStatus::Unknown,
        })
    }

    /// Drops the queued or parked row `id` of the account's queue and
    /// returns the action kind it held.
    ///
    /// Dropping a row is the store owner's write, so this takes that role
    /// for the call and fails `store-owned` while a sync holds it.
    pub(crate) fn cancel_queued(&self, id: i64) -> Result<String> {
        let kind = match self.queue_row(id)? {
            PimdirActionStatus::Pending { kind, .. }
            | PimdirActionStatus::Parked { kind, .. }
            | PimdirActionStatus::Unconfirmed { kind, .. } => kind,
            PimdirActionStatus::Applied { .. } => bail!("Row {id} was applied already"),
            // NOTE: a bare row id states no store, so io-pimdir follows it
            // unchecked and never answers a reset for it.
            PimdirActionStatus::Unknown | PimdirActionStatus::Reset => bail!(
                "No queued action with row {id} in this account's queue; it may have been \
                 synced already"
            ),
        };

        let root = &self.client.root;
        let mut owner = PimdirStore::open_current(root).map_err(|err| match err {
            PimdirError::Owned(_) => CodedError::new(
                ErrorCode::StoreOwned,
                format!(
                    "A sync is running on `{}`, so the queue cannot be edited; the action may \
                     have been applied already",
                    root.display(),
                ),
            )
            .into(),
            err => store_error(err, format!("Cancel queued action {id}")),
        })?;

        let dropped = owner
            .drop_action(id)
            .map_err(|err| store_error(err, format!("Cancel queued action {id}")))?;

        if !dropped {
            bail!("No queued action with row {id}; it may have been synced already");
        }

        Ok(kind)
    }
}

/// The title a calendar summary carries, `None` when empty or when the
/// summary is no calendar one.
fn title(summary: PimdirSummary) -> Option<String> {
    let title = match summary {
        PimdirSummary::Event(event) => event.summary,
        PimdirSummary::Task(task) => task.summary,
        PimdirSummary::Journal(journal) => journal.summary,
        _ => return None,
    };

    (!title.is_empty()).then_some(title)
}

#[cfg(test)]
mod tests {
    use io_pimdir::client::{PimdirStore, producer::PimdirActionStatus};

    use crate::{
        config::PimdirConfig,
        error::{self, ErrorCode},
        pimdir::backend::PimdirBackend,
    };

    const EVENT: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//t//t//EN\r\n\
        BEGIN:VEVENT\r\nUID:standup@example.org\r\nDTSTAMP:20260101T000000Z\r\n\
        DTSTART:20261120T090000Z\r\nSUMMARY:Standup\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

    /// A store holding the calendars `cal` and `other`, one event queued
    /// for creation in `cal`, and a backend over it.
    fn store() -> (tempfile::TempDir, PimdirBackend) {
        let dir = tempfile::tempdir().unwrap();
        let store = PimdirStore::open(dir.path()).unwrap();
        store.ensure_collection("cal", "text/calendar").unwrap();
        store.ensure_collection("other", "text/calendar").unwrap();
        drop(store);

        let config = PimdirConfig {
            root: dir.path().to_path_buf(),
            account: None,
        };
        let mut backend = PimdirBackend::new(config).unwrap();
        backend
            .create_item("cal", EVENT.as_bytes().to_vec())
            .unwrap();

        (dir, backend)
    }

    /// A queued creation lists, in its calendar or every one, by its row
    /// with the link id `event add` reported and the summary of the event
    /// it carries, shows `pending`, and once cancelled leaves the queue,
    /// its row then unknown.
    #[test]
    fn a_queued_creation_lists_shows_and_cancels() {
        let (_dir, backend) = store();

        let rows = backend.queued_rows(None).unwrap();
        assert_eq!(rows.len(), 1, "{rows:?}");
        assert!(backend.queued_rows(Some("other")).unwrap().is_empty());
        assert!(backend.parked_rows(None).unwrap().is_empty());

        let row = &rows[0];
        assert_eq!(row.collection, "cal");
        assert_eq!(row.kind, "add");
        assert_eq!(row.link_id.as_deref(), Some("standup@example.org"));
        assert_eq!(row.item_id, None);
        assert_eq!(row.summary.as_deref(), Some("Standup"));
        let row = row.id;

        let status = backend.queue_row(row).unwrap();
        assert!(
            matches!(status, PimdirActionStatus::Pending { ref kind, .. } if kind == "add"),
            "{status:?}"
        );

        assert_eq!(backend.cancel_queued(row).unwrap(), "add");
        assert!(backend.queued_rows(Some("cal")).unwrap().is_empty());
        assert_eq!(backend.queue_row(row).unwrap(), PimdirActionStatus::Unknown);

        let err = backend.cancel_queued(row).unwrap_err();
        assert!(err.to_string().contains("No queued action"), "{err}");
    }

    /// The queue of a calendar the store does not hold is refused
    /// `calendar-not-found`, rather than listed empty.
    #[test]
    fn an_unknown_calendar_has_no_queue() {
        let (_dir, backend) = store();

        let err = backend.queued_rows(Some("nope")).unwrap_err();
        assert_eq!(error::code(&err), Some(ErrorCode::CalendarNotFound));
    }
}
