//! # Pimdir queue
//!
//! The `pimdir queue` commands, over the writes and intents staged in the
//! store's queue for its owner to apply or perform, as Himalaya's: list
//! the rows staged in the account's calendars, show where one stands,
//! cancel one.

pub mod cancel;
pub mod list;
pub mod show;

use anyhow::{Error, Result, anyhow};
use clap::Subcommand;
use io_pimdir::{
    client::{
        PimdirError, PimdirStore,
        producer::{PimdirActionStatus, PimdirPendingAction},
    },
    codec::{PimdirAction, PimdirActionHandle, PimdirStoreIdentity, action_from_payload},
    intent::{PimdirIntentItem, PimdirInvitation},
    object::PimdirHash,
    summary::{PimdirSummary, calendar::derive},
};
use pimalaya_cli::{error::CodedError, printer::Printer};

use crate::{
    error::{ErrorCode, ItemError},
    pimdir::{
        backend::PimdirBackend,
        client::store_error,
        queue::{
            cancel::PimdirQueueCancelCommand,
            list::{PimdirQueueListCommand, PimdirQueueRow},
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
    /// The store's identity, its id and incarnation (pimdir STORAGE §4.5),
    /// which a listed row's handle names.
    pub(crate) fn store_identity(&self) -> Result<PimdirStoreIdentity> {
        let cursor = self
            .client
            .reader
            .change_cursor()
            .map_err(|err| store_error(err, "Read the pimdir store identity"))?;

        Ok(PimdirStoreIdentity {
            store_id: cursor.store_id,
            incarnation: cursor.incarnation,
        })
    }

    /// The handle `given` names under this store: as given when it names a
    /// store, a bare id named under this store's identity, as `pimdir queue
    /// list` prints its handle.
    pub(crate) fn named(&self, given: &PimdirActionHandle) -> Result<PimdirActionHandle> {
        if given.store.is_some() {
            return Ok(given.clone());
        }

        Ok(PimdirActionHandle {
            id: given.id,
            store: Some(self.store_identity()?),
        })
    }

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

    /// The pending rows of every kind in `calendar`, or in every calendar
    /// of the account, in append order, and an item error for each
    /// calendar whose queue holds a row that does not decode, whose rows
    /// are then left out.
    pub(crate) fn pending_rows(
        &self,
        calendar: Option<&str>,
    ) -> Result<(Vec<PimdirQueueRow>, Vec<ItemError>)> {
        let identity = self.store_identity()?;
        let mut rows = Vec::new();
        let mut errors = Vec::new();

        for collection in self.queue_calendars(calendar)? {
            let pending = match self.client.reader.pending_actions(&collection) {
                Ok(pending) => pending,
                Err(err @ PimdirError::Action(_)) => {
                    let context = format!("Decode the pimdir queue of calendar `{collection}`");
                    errors.push(ItemError::new(None, &store_error(err, context)));
                    continue;
                }
                Err(err) => return Err(store_error(err, "Read the pimdir queue")),
            };

            for queued in pending {
                rows.push(self.pending_row(&identity, queued));
            }
        }

        rows.sort_by_key(|row| row.row);
        Ok((rows, errors))
    }

    /// The queue row `handle` names, pending or parked in `collection`, as
    /// `pimdir queue list` prints it, refused `row-not-found` when the
    /// queue holds it no more.
    pub(crate) fn listed_row(
        &self,
        handle: &PimdirActionHandle,
        collection: &str,
    ) -> Result<PimdirQueueRow> {
        let parked = self.parked_rows(Some(collection))?;

        if let Some(row) = parked.into_iter().find(|row| row.row == handle.id) {
            return Ok(row);
        }

        let identity = self.store_identity()?;
        let pending = self
            .client
            .reader
            .pending_actions(collection)
            .map_err(|err| store_error(err, format!("Read queue row {handle}")))?;

        match pending.into_iter().find(|queued| queued.id == handle.id) {
            Some(queued) => Ok(self.pending_row(&identity, queued)),
            None => Err(row_not_found(handle)),
        }
    }

    /// The listed row of the pending action `queued`.
    fn pending_row(
        &self,
        identity: &PimdirStoreIdentity,
        queued: PimdirPendingAction,
    ) -> PimdirQueueRow {
        let addressed = self.addressed(&queued.collection, &queued.action);

        PimdirQueueRow {
            row: queued.id,
            handle: handle(identity, queued.id),
            kind: queued.action.kind().to_owned(),
            collection: queued.collection,
            link_id: addressed.link_id,
            item_id: addressed.item_id,
            producer: queued.producer,
            queued_at: queued.created_at,
            attempts: u64::try_from(queued.attempts).unwrap_or_default(),
            unconfirmed: false,
            error: None,
            summary: addressed.summary,
        }
    }

    /// The summary of the event `seq` of `collection`, `None` when the
    /// store holds no such event or it has none.
    pub(crate) fn summary(&self, collection: &str, seq: i64) -> Option<String> {
        let item = self.item(collection, &seq.to_string()).ok()?;
        title(item.summary?)
    }

    /// The rows the store's owner parked in `calendar`, or in every
    /// calendar of the account, in append order.
    pub(crate) fn parked_rows(&self, calendar: Option<&str>) -> Result<Vec<PimdirQueueRow>> {
        let calendars = self.queue_calendars(calendar)?;
        let identity = self.store_identity()?;

        Ok(self
            .client
            .reader
            .parked_actions()
            .map_err(|err| store_error(err, "Read the pimdir queue"))?
            .into_iter()
            .filter(|parked| calendars.contains(&parked.collection))
            .map(|parked| {
                // NOTE: a payload the store parked for not decoding
                // addresses nothing calendula can name.
                let addressed = action_from_payload(&parked.action, &parked.payload)
                    .map(|action| self.addressed(&parked.collection, &action))
                    .unwrap_or_default();

                PimdirQueueRow {
                    row: parked.id,
                    handle: handle(&identity, parked.id),
                    unconfirmed: parked.unconfirmed().is_some(),
                    kind: parked.action,
                    collection: parked.collection,
                    link_id: addressed.link_id,
                    item_id: addressed.item_id,
                    producer: parked.producer,
                    queued_at: parked.created_at,
                    attempts: u64::try_from(parked.attempts).unwrap_or_default(),
                    error: Some(parked.error),
                    summary: addressed.summary,
                }
            })
            .collect())
    }

    /// What a row of `collection` holding `action` addresses: the item it
    /// creates by its link id, the stored item it acts on by its id, and
    /// the summary of the body it pins, else of the item it addresses.
    fn addressed(&self, collection: &str, action: &PimdirAction) -> Addressed {
        let pinned = |hash: &PimdirHash| {
            let body = self.client.blobs.get(hash).ok().flatten()?;
            title(derive(&body).summary?)
        };

        let stored = |seq: i64| {
            let id = seq.to_string();
            let item = self.item(collection, &id).ok();
            let summary = item.and_then(|item| title(item.summary?));

            Addressed {
                item_id: Some(id),
                summary,
                ..Default::default()
            }
        };

        match action {
            PimdirAction::Add {
                link_id, object, ..
            } => Addressed {
                link_id: link_id.as_ref().map(|id| id.0.clone()),
                summary: object.as_ref().and_then(pinned),
                ..Default::default()
            },
            PimdirAction::Update { seq, object, .. } => {
                let addressed = stored(*seq);

                Addressed {
                    summary: pinned(object).or(addressed.summary),
                    ..addressed
                }
            }
            PimdirAction::SetFlags { seq, .. }
            | PimdirAction::SetProperty { seq, .. }
            | PimdirAction::Remove { seq, .. }
            | PimdirAction::Move { seq, .. }
            | PimdirAction::Copy { seq, .. } => stored(*seq),
            PimdirAction::SetPerformer { .. } => Addressed::default(),
            PimdirAction::Unknown { kind, payload, .. } => {
                match PimdirInvitation::from_payload(kind, payload).map(|it| it.item) {
                    Ok(PimdirIntentItem::Seq(seq)) => stored(seq),
                    Ok(PimdirIntentItem::Link(link_id)) => Addressed {
                        link_id: Some(link_id.0),
                        ..Default::default()
                    },
                    Err(_) => Addressed::default(),
                }
            }
        }
    }

    /// Where the queue row `handle` names stands, `Unknown` for a row of a
    /// calendar another account holds, `Reset` for a handle another store,
    /// or another incarnation of this one, answered (pimdir STORAGE §4.5).
    pub(crate) fn queue_row(&self, handle: &PimdirActionHandle) -> Result<PimdirActionStatus> {
        let status = self
            .client
            .reader
            .action_status(handle.clone())
            .map_err(|err| store_error(err, format!("Read queue row {handle}")))?;

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

    /// Drops the queued or parked row `handle` names of the account's
    /// queue and returns the action kind it held.
    ///
    /// Dropping a row is the store owner's write, so this takes that role
    /// for the call and fails `store-owned` while a sync holds it.
    pub(crate) fn cancel_queued(&self, handle: &PimdirActionHandle) -> Result<String> {
        let kind = match self.queue_row(handle)? {
            PimdirActionStatus::Pending { kind, .. }
            | PimdirActionStatus::Parked { kind, .. }
            | PimdirActionStatus::Unconfirmed { kind, .. } => kind,
            status => return Err(uncancellable(handle, &status)),
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
            err => store_error(err, format!("Cancel queued action {handle}")),
        })?;

        let dropped = owner
            .drop_action(handle.clone())
            .map_err(|err| store_error(err, format!("Cancel queued action {handle}")))?;

        if !dropped {
            return Err(row_not_found(handle));
        }

        Ok(kind)
    }
}

/// What a queue row addresses, each `None` when it addresses none.
#[derive(Debug, Default)]
struct Addressed {
    /// The link id of the item the row creates.
    link_id: Option<String>,
    /// The id of the stored item the row acts on.
    item_id: Option<String>,
    /// The summary of the body the row pins, else of the item.
    summary: Option<String>,
}

/// Why queue row `row`, standing at `status`, cannot be cancelled nor
/// shown as a queued row: applied already, queued in another store than
/// this one under `store-reset`, or not a row of the account's queue under
/// `row-not-found`. A bare row id states no store, so io-pimdir follows it
/// unchecked and never answers a reset for it.
pub(crate) fn uncancellable(row: &PimdirActionHandle, status: &PimdirActionStatus) -> Error {
    match status {
        PimdirActionStatus::Applied { .. } => anyhow!("Row {row} was applied already"),
        PimdirActionStatus::Reset => {
            let message = format!(
                "Row {row} was queued in another store: a copy restored since, where \
                 its id may name another row"
            );
            CodedError::new(ErrorCode::StoreReset, message).into()
        }
        _ => row_not_found(row),
    }
}

/// The refusal of a queue row the account's queue does not hold.
pub(crate) fn row_not_found(row: &PimdirActionHandle) -> Error {
    let message = format!(
        "No queued action with row {row} in this account's queue; it may have been synced already"
    );
    CodedError::new(ErrorCode::RowNotFound, message).into()
}

/// The handle of the row `id` of the store `identity`, as `pimdir queue
/// show` and `cancel` take it.
pub(crate) fn handle(identity: &PimdirStoreIdentity, id: i64) -> String {
    PimdirActionHandle {
        id,
        store: Some(identity.clone()),
    }
    .to_string()
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
        shared::item::ItemOrigin,
    };

    const EVENT: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//t//t//EN\r\n\
        BEGIN:VEVENT\r\nUID:standup@example.org\r\nDTSTAMP:20260101T000000Z\r\n\
        DTSTART:20261120T090000Z\r\nSUMMARY:Standup\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

    /// A store holding the calendars `cal` and `other`, one event queued
    /// for creation in `cal`, and a backend over it.
    fn store() -> (crate::pimdir::client::StoreDir, PimdirBackend) {
        let dir = crate::pimdir::client::store_dir();
        let store = PimdirStore::open(dir.path()).unwrap();
        store.ensure_collection("cal", "text/calendar").unwrap();
        store.ensure_collection("other", "text/calendar").unwrap();
        drop(store);
        for collection in ["cal", "other"] {
            crate::pimdir::client::sync_from(dir.path(), None, collection);
        }

        let config = PimdirConfig {
            root: dir.path().to_path_buf(),
            account: None,
        };
        let mut backend = PimdirBackend::new(config).unwrap();
        backend
            .create_item("cal", EVENT.as_bytes().to_vec(), ItemOrigin::User)
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

        let rows = backend.pending_rows(None).unwrap().0;
        assert_eq!(rows.len(), 1, "{rows:?}");
        assert!(backend.pending_rows(Some("other")).unwrap().0.is_empty());
        assert!(backend.parked_rows(None).unwrap().is_empty());

        let row = &rows[0];
        assert_eq!(row.collection, "cal");
        assert_eq!(row.kind, "add");
        assert_eq!(row.link_id.as_deref(), Some("standup@example.org"));
        assert_eq!(row.item_id, None);
        assert_eq!(row.summary.as_deref(), Some("Standup"));
        let row: io_pimdir::codec::PimdirActionHandle = row.handle.parse().unwrap();
        assert_eq!(row.store, Some(backend.store_identity().unwrap()));

        let status = backend.queue_row(&row).unwrap();
        assert!(
            matches!(status, PimdirActionStatus::Pending { ref kind, .. } if kind == "add"),
            "{status:?}"
        );

        assert_eq!(backend.cancel_queued(&row).unwrap(), "add");
        assert!(backend.pending_rows(Some("cal")).unwrap().0.is_empty());
        assert_eq!(
            backend.queue_row(&row).unwrap(),
            PimdirActionStatus::Unknown
        );

        let err = backend.cancel_queued(&row).unwrap_err();
        assert!(err.to_string().contains("No queued action"), "{err}");
        assert_eq!(error::code(&err), Some(ErrorCode::RowNotFound), "{err}");
    }

    /// A row named by the handle its enqueue answered is followed and
    /// cancelled in the store that answered it, and refused `store-reset`,
    /// nothing dropped, under another store id or incarnation: a restored
    /// copy, where the id may name another row (pimdir STORAGE §4.5).
    #[test]
    fn a_handle_of_another_store_is_refused_as_a_reset() {
        use io_pimdir::{
            codec::{PimdirAction, PimdirActionHandle},
            placement::{PimdirFlags, PimdirLinkId},
        };

        let (_dir, backend) = store();
        let body = EVENT.replace("standup@", "retro@");
        let handle = backend
            .client
            .producer()
            .unwrap()
            .enqueue_user_body("cal", body.as_bytes(), |hash| PimdirAction::Add {
                link_id: Some(PimdirLinkId("retro@example.org".into())),
                flags: PimdirFlags::default(),
                object: Some(hash),
                supersedes: None,
            })
            .unwrap()
            .handle;
        let identity = handle.store.clone().unwrap();

        let mut restored = identity.clone();
        restored.incarnation += 1;
        let mut foreign = identity.clone();
        foreign.store_id = "0".repeat(32);

        for store in [restored, foreign] {
            let other = PimdirActionHandle {
                id: handle.id,
                store: Some(store),
            };
            assert_eq!(
                backend.queue_row(&other).unwrap(),
                PimdirActionStatus::Reset
            );
            let err = backend.cancel_queued(&other).unwrap_err();
            assert_eq!(error::code(&err), Some(ErrorCode::StoreReset), "{err}");
        }
        assert_eq!(backend.pending_rows(Some("cal")).unwrap().0.len(), 2);

        let parsed: PimdirActionHandle = handle.to_string().parse().unwrap();
        assert_eq!(backend.cancel_queued(&parsed).unwrap(), "add");
        assert_eq!(backend.pending_rows(Some("cal")).unwrap().0.len(), 1);
    }

    /// A bare row id is named under the store's identity, the handle
    /// `queue list` prints for it, and a handle is kept as given, another
    /// store's included, so `queue show` and `cancel` print the row as it
    /// was named.
    #[test]
    fn a_row_is_named_under_the_store_identity() {
        use io_pimdir::codec::PimdirActionHandle;

        let (_dir, backend) = store();
        let listed = backend.pending_rows(None).unwrap().0.remove(0).handle;
        let given: PimdirActionHandle = listed.parse().unwrap();

        let bare = PimdirActionHandle {
            id: given.id,
            store: None,
        };
        assert_eq!(backend.named(&bare).unwrap().to_string(), listed);
        assert_eq!(backend.named(&given).unwrap(), given);

        let mut restored = given.clone();
        restored.store.as_mut().unwrap().incarnation += 1;
        assert_eq!(backend.named(&restored).unwrap(), restored);
    }

    /// The queue of a calendar the store does not hold is refused
    /// `calendar-not-found`, rather than listed empty.
    #[test]
    fn an_unknown_calendar_has_no_queue() {
        let (_dir, backend) = store();

        let err = backend.pending_rows(Some("nope")).unwrap_err();
        assert_eq!(error::code(&err), Some(ErrorCode::CalendarNotFound));
    }

    /// A pending row that does not decode is an item error naming its
    /// calendar, coded `action-malformed`, the other calendars still
    /// listed, rather than a failure of the whole listing.
    #[test]
    fn an_undecodable_pending_row_is_an_item_error() {
        let (dir, backend) = store();
        drop(backend);

        let db = rusqlite::Connection::open(dir.path().join("pimdir.db")).unwrap();
        let sql = "UPDATE queue SET payload = replace(payload, '\"v\":1', '\"w\":1')";
        assert_eq!(db.execute(sql, []).unwrap(), 1);
        drop(db);

        let config = PimdirConfig {
            root: dir.path().to_path_buf(),
            account: None,
        };
        let backend = PimdirBackend::new(config).unwrap();
        let (rows, errors) = backend.pending_rows(None).unwrap();
        assert!(rows.is_empty(), "{rows:?}");
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert_eq!(errors[0].code, Some(ErrorCode::ActionMalformed));
        assert!(errors[0].error.contains("`cal`"), "{errors:?}");
    }
}
