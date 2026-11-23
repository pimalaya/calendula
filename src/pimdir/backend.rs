//! # Pimdir backend
//!
//! The shared-API adapter over a pimdir store, projecting its items
//! through [`io_pimdir`]'s reader plus the blob store.
//!
//! An item whose body is not local still lists, carrying no bytes; only
//! a read of it reports "body not fetched", the cue to sync rather than
//! a data-loss error. Its kind and its date answer from the typed
//! summary the store keeps beside the row (pimdir STORAGE Annex A).
//!
//! Writes append one action to the store's queue (pimdir STORAGE §15.1):
//! the producer writes the body and the row pinning it in one step under
//! its staging lock (§8), once the store's checks passed, so a refused
//! write leaves no body behind. The owner, a sync, derives the summary
//! from that body, applies and pushes it, while the reader folds the
//! pending queue over its reads so a staged change shows before that.
//!
//! Calendars come from the sync, so the collection verbs (create,
//! update, delete) are not served here: a cache does not invent
//! collections its source does not have.
//!
//! Ids are the store's public `seq`, stable across every collection an
//! item is filed in, never the internal link id.

use std::{ops::ControlFlow, path::PathBuf};

use anyhow::{Result, anyhow, bail};
use ical::component::IcalComponentKind;
use io_pimdir::{
    capability::{
        CALENDAR_CANCEL, CALENDAR_CANCEL_OCCURRENCE, CALENDAR_COMPONENT_VTODO,
        CALENDAR_ONLINE_MEETING, CALENDAR_REPLY, CALENDAR_REPLY_OCCURRENCE, CALENDAR_SCHEDULING,
        PimdirRefusal, PimdirSourceCapabilities, PimdirSupport,
    },
    client::{
        PimdirDuplicate, PimdirError,
        producer::{PimdirEnqueued, PimdirProducer},
        reader::{PimdirCollection, PimdirItem},
    },
    codec::{PimdirAction, PimdirActionHandle},
    collection::media_type,
    intent::{PimdirIntentItem, PimdirInvitation, PimdirPartstat},
    object::PimdirHash,
    placement::PimdirFlags,
    summary::{
        PimdirSummary,
        calendar::{PimdirTime, derive, online_meeting},
    },
};
use log::warn;
use pimalaya_cli::error::CodedError;

use crate::{
    account::context::Account,
    cli::resolve_account,
    config::PimdirConfig,
    error::{self, ErrorCode, ItemError, calendar_duplicate, calendar_not_found, item_not_found},
    pimdir::{
        client::{PimdirClient, store_error},
        status::{
            PimdirCalendarStatus, PimdirParkedStatus, PimdirRefusedStatus, PimdirStatusOutput,
        },
    },
    shared::{
        calendar::{Calendar, CalendarDiff},
        event::{Event, expand},
        ical::may_hold_kind,
        item::{
            CalendarItem, CalendarItemList, CalendarItemQuery, CalendarItemWindowed,
            CalendarTimeRange, ItemOrigin,
        },
    },
};

/// The media type a pimdir collection carries to be a calendar.
pub(crate) const CALENDAR_KIND: &str = "text/calendar";

/// How many items to pull per keyset page when scanning a collection.
#[cfg(not(test))]
const SCAN_BATCH: usize = 500;

/// A batch small enough for a test store to span several.
#[cfg(test)]
const SCAN_BATCH: usize = 7;

/// The shared-API glue over a pimdir store.
pub struct PimdirBackend {
    pub(crate) client: PimdirClient,
    /// The account's settings, default when built from no account.
    pub(crate) account: Account,
    /// The name of the account, empty when built from no account.
    pub(crate) name: String,
}

impl PimdirBackend {
    /// Opens the store at the configured root.
    pub fn new(config: PimdirConfig) -> Result<Self> {
        Ok(Self {
            client: PimdirClient::new(config)?,
            account: Account::default(),
            name: String::new(),
        })
    }

    /// Loads the configuration, picks the active account, then opens
    /// the store.
    ///
    /// Bails when the account carries no `[pimdir]` block.
    pub fn build(config_paths: &[PathBuf], account_name: Option<&str>) -> Result<Self> {
        let (config, name, mut account_config) = resolve_account(config_paths, account_name)?;

        let pimdir_config = account_config
            .pimdir
            .take()
            .ok_or_else(|| anyhow!("pimdir configuration is missing for account `{name}`"))?;

        let account = Account::from(config).merge(Account::from(account_config));

        let mut backend = Self::new(pimdir_config)?;
        backend.account = account;
        backend.name = name;
        Ok(backend)
    }

    /// The calendar a `pimdir` command operates on: the `-k/--calendar`
    /// flag, then `calendar.default`, else coded `calendar-default-missing`.
    pub fn calendar_id(&self, flag: Option<String>) -> Result<String> {
        self.account.calendar_id(flag)
    }

    /// Lists the calendar collections.
    ///
    /// Those declaring `text/calendar`, plus the kind-less ones a sync
    /// created before any consumer declared a kind.
    pub fn list_calendars(&mut self) -> Result<Vec<Calendar>> {
        let mut calendars: Vec<Calendar> = self
            .calendar_collections()?
            .into_iter()
            .map(|collection| Calendar {
                name: if collection.name.is_empty() {
                    collection.id.clone()
                } else {
                    collection.name
                },
                id: collection.id,
                description: collection.description,
                color: collection.color,
                default: collection.role.as_deref() == Some("default"),
            })
            .collect();

        calendars.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(calendars)
    }

    /// Refuses a create naming the calendar `id` the store holds already,
    /// coded `calendar-duplicate`.
    pub fn check_new_calendar(&self, id: &str) -> Result<()> {
        if self
            .calendar_collections()?
            .iter()
            .any(|held| held.id == id)
        {
            return Err(calendar_duplicate(id));
        }

        Ok(())
    }

    /// Refuses to create a calendar.
    ///
    /// Declaring a collection is an owner write (pimdir STORAGE §8) and
    /// this backend is a producer, and a collection no sync knows about
    /// is one no sync would carry.
    pub fn create_calendar(
        &mut self,
        _id: Option<&str>,
        _name: &str,
        _description: Option<&str>,
        _color: Option<&str>,
    ) -> Result<String> {
        bail!(unsupported("create"))
    }

    /// Refuses to update a calendar, for the same reason as
    /// [`create_calendar`](Self::create_calendar).
    pub fn update_calendar(&mut self, _id: &str, _patch: CalendarDiff) -> Result<()> {
        bail!(unsupported("update"))
    }

    /// Refuses to delete a calendar, for the same reason as
    /// [`create_calendar`](Self::create_calendar).
    pub fn delete_calendar(&mut self, _id: &str) -> Result<()> {
        bail!(unsupported("delete"))
    }

    /// Lists a collection's items, reading each local body.
    ///
    /// An unhydrated item lists with empty contents, and a range still
    /// filters it off the stored summary: a cache hiding its own
    /// undownloaded items from a window would answer another question.
    /// So does one whose blob is gone or fails its name, not hydrated
    /// either (pimdir GUIDE §14). One whose body cannot be read lists with
    /// empty contents too, beside an `item-unreadable` entry kept with it
    /// in the window.
    ///
    /// The collection streams through in keyset batches, one body held
    /// at a time: what the listing keeps is the window, and the scan stops
    /// once the window is full. A body is read only to answer the filters
    /// or to be returned, never for an item the stored summary already
    /// rules out (see [`ruled_out`]).
    pub fn list_items(
        &mut self,
        calendar_id: &str,
        query: CalendarItemQuery<'_>,
    ) -> Result<CalendarItemList> {
        self.known_collection(calendar_id)?;

        let mut page = Window::new(query.skip, query.take);
        if page.is_closed() {
            return Ok(CalendarItemList::default());
        }

        // NOTE: a hint leaving items out would shift a window.
        let uid = query.uid.filter(|_| query.take.is_none());
        let filtered = query.kind.is_some() || query.range.is_some();
        let restaged = match query.kind.is_some() || uid.is_some() {
            true => self.restaged(calendar_id)?,
            false => Some(Vec::new()),
        };

        self.scan(calendar_id, |stored| {
            if let Some(restaged) = &restaged
                && ruled_out(&stored, query.kind, uid, restaged)
            {
                return Ok(ControlFlow::Continue(()));
            }

            // NOTE: with no filter to answer, an item before the page
            // needs no body to be counted.
            if !filtered && page.skips() {
                return Ok(ControlFlow::Continue(()));
            }

            let (mut item, error) = self.item_from(calendar_id, &stored);

            if let Some(kind) = query.kind
                && !is_kind(&item, &stored, kind)
            {
                return Ok(ControlFlow::Continue(()));
            }

            if let Some(range) = query.range
                && !in_range(&mut item, &stored, range)
            {
                return Ok(ControlFlow::Continue(()));
            }

            if filtered && page.skips() {
                return Ok(ControlFlow::Continue(()));
            }

            Ok(page.keep(item, error))
        })?;

        Ok(page.into_list())
    }

    /// Reads one item's bytes from its content-addressed blob.
    ///
    /// Fails with a clear "body not fetched" when the item is not
    /// hydrated: a state to resolve with a sync, not a missing item.
    pub fn get_item(&mut self, calendar_id: &str, item_id: &str) -> Result<CalendarItem> {
        self.known_collection(calendar_id)?;

        let stored = self.item(calendar_id, item_id)?;

        let Some(contents) = self.body(calendar_id, &stored)? else {
            return Err(body_pending(item_id, calendar_id));
        };

        Ok(CalendarItem {
            id: stored.seq.to_string(),
            calendar_id: calendar_id.to_owned(),
            etag: stored.object.map(|hash| hash.0),
            contents,
            stand_in: None,
            windowed: None,
        })
    }

    /// Stages a locally-authored item as an `add` the next sync pushes.
    ///
    /// Returns the item's link id, its `UID` under the store's own
    /// derivation (pimdir STORAGE Annex A.3): a queued create carries no
    /// public `seq` until the store's owner applies it, so there is no
    /// store-assigned id to report yet. A link id the calendar holds live
    /// or has queued is refused by the store, nothing queued.
    ///
    /// The action states a `UID` key and no `hash:` one: the store derives
    /// that one from the body it holds, a [`ItemOrigin::Foreign`] body's
    /// `X-PIMDIR-` properties taken out, and reports it.
    pub fn create_item(
        &mut self,
        calendar_id: &str,
        contents: Vec<u8>,
        origin: ItemOrigin,
    ) -> Result<String> {
        self.known_collection(calendar_id)?;
        self.check_synced(calendar_id)?;
        if origin == ItemOrigin::User {
            self.check_online_meeting(calendar_id, None, None, &contents)?;
        }

        let derived = derive(&contents).link_id;
        let stated = (!derived.as_str().starts_with("hash:")).then_some(derived);

        let mut producer = self.client.producer()?;
        let enqueued = enqueue_with_body(&mut producer, calendar_id, &contents, origin, |hash| {
            PimdirAction::Add {
                link_id: stated,
                flags: PimdirFlags::default(),
                object: Some(hash),
                supersedes: None,
            }
        })?;

        let link_id = enqueued
            .link_id
            .ok_or_else(|| anyhow!("The pimdir store reported no key for the queued item"))?;

        Ok(link_id.0)
    }

    /// Stages a body replacement as an `update` the next sync pushes.
    ///
    /// `if_match` gates the staging on the item's version, the store's
    /// hash of its body with the pending queue folded in. The action then
    /// names that body as the one it replaces (pimdir STORAGE §15.3), so
    /// the owner parks it, rather than applies it, when a sync replaced
    /// the body while it waited in the queue. Without `if_match` no body
    /// was read, and the edit applies over whatever the item holds. The
    /// engine still three-way merges the update against the base body it
    /// recorded at sync time: the gate is the caller's, the merge the
    /// store's.
    pub fn update_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
        origin: ItemOrigin,
    ) -> Result<()> {
        self.known_collection(calendar_id)?;
        self.check_synced(calendar_id)?;

        let stored = self.item(calendar_id, item_id)?;
        let current = self.local_body(&stored)?;
        let held = current.as_ref().and(stored.object.clone());
        check_version(held.as_ref(), calendar_id, item_id, if_match)?;
        if origin == ItemOrigin::User {
            self.check_online_meeting(
                calendar_id,
                Some(stored.seq),
                current.as_deref(),
                &contents,
            )?;
        }

        // NOTE: `check_version` passed, so `if_match` names the body the
        // item holds.
        let read = if_match.and(held);

        let mut producer = self.client.producer()?;
        enqueue_with_body(&mut producer, calendar_id, &contents, origin, |hash| {
            PimdirAction::Update {
                seq: stored.seq,
                object: hash,
                if_match: read,
                resolves: None,
            }
        })?;

        Ok(())
    }

    /// Refuses, before anything is queued, a write to a calendar no
    /// source of the store syncs: nobody would push it (TWN-3).
    fn check_synced(&self, calendar_id: &str) -> Result<()> {
        let sources = self
            .client
            .reader
            .capabilities(calendar_id)
            .map_err(|err| store_error(err, format_args!("Read the sources of `{calendar_id}`")))?;

        if sources.is_empty() {
            return Err(CodedError::new(
                ErrorCode::NoPerformer,
                format!(
                    "No source syncs calendar `{calendar_id}` yet, so none would push this \
                     write: run a sync first"
                ),
            )
            .into());
        }

        Ok(())
    }

    /// Refuses a write asking for an online meeting no source concerned
    /// can create (pimdir STORAGE Annex B.1, §15.6), before its body is
    /// staged.
    ///
    /// The sources concerned are those syncing the calendar for a create,
    /// and those binding the item `seq`, the ones that push it, for an
    /// update. The store's own gate refuses a declared source lacking
    /// `calendar.online-meeting`, and lets a store whose sources declare
    /// nothing through: its owner would write the ask and nobody would
    /// act on it. So the ask passes only where a source declares the
    /// capability, fully or in part; one the event already carried is
    /// no new ask, and passes as the store's gate lets it.
    fn check_online_meeting(
        &self,
        calendar_id: &str,
        seq: Option<i64>,
        current: Option<&[u8]>,
        new: &[u8],
    ) -> Result<()> {
        if !online_meeting(new) || current.is_some_and(online_meeting) {
            return Ok(());
        }

        let sources = match seq {
            Some(seq) => self.client.reader.item_capabilities(calendar_id, seq),
            None => self.client.reader.capabilities(calendar_id),
        }
        .map_err(|err| {
            store_error(
                err,
                format_args!("Read the capabilities of `{calendar_id}`"),
            )
        })?;

        let support = |source: &PimdirSourceCapabilities| {
            let declared = source.declared.as_ref()?;
            declared
                .get(CALENDAR_ONLINE_MEETING)
                .map(|(support, _)| *support)
        };

        if sources.iter().any(|source| {
            matches!(
                support(source),
                Some(PimdirSupport::Full | PimdirSupport::Partial)
            )
        }) {
            return Ok(());
        }

        let lacking: Vec<&str> = sources
            .iter()
            .filter(|source| source.declared.is_some())
            .map(|source| source.source.as_str())
            .collect();

        Err(staging_error(PimdirError::Unsupported(PimdirRefusal {
            capability: CALENDAR_ONLINE_MEETING.to_owned(),
            source: lacking.join(", "),
            detail: None,
        })))
    }

    /// Stages a `remove` action: a tombstone, then a server-side delete.
    ///
    /// `if_match` gates it as it gates [`update_item`](Self::update_item),
    /// and the action names that body as the one it deletes (pimdir
    /// STORAGE §15.3), so the owner parks it when a sync replaced the body
    /// while it waited in the queue. Without `if_match` no body was read,
    /// and the remove deletes whatever the item holds.
    ///
    /// It states no `resolves`, calendula deciding no conflict: the owner
    /// parks a remove meeting one.
    pub fn delete_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        if_match: Option<&str>,
    ) -> Result<()> {
        self.known_collection(calendar_id)?;
        self.check_synced(calendar_id)?;

        let stored = self.item(calendar_id, item_id)?;
        let held = match self.local_body(&stored)? {
            Some(_) => stored.object.clone(),
            None => None,
        };
        check_version(held.as_ref(), calendar_id, item_id, if_match)?;

        // NOTE: `check_version` passed, so `if_match` names the body the
        // item holds.
        let action = PimdirAction::Remove {
            seq: stored.seq,
            if_match: if_match.and(held),
            resolves: None,
        };

        let mut producer = self.client.producer()?;
        let enqueued = producer
            .enqueue(calendar_id, &action, None)
            .map_err(staging_error)?;

        for partial in &enqueued.partial {
            warn!("{partial}");
        }

        Ok(())
    }

    /// Collects the accounts, the per-calendar hydration state, the parked
    /// actions and the refusals standing that `pimdir status` reports.
    pub fn status(&mut self) -> Result<PimdirStatusOutput> {
        let accounts = self.client.reader.list_accounts()?;
        let mut calendars = Vec::new();
        let mut refused = Vec::new();

        for collection in self.calendar_collections()? {
            let refusals = self.client.reader.refusals(&collection.id)?;
            refused.extend(refusals.into_iter().map(|row| PimdirRefusedStatus {
                calendar_id: collection.id.clone(),
                item_id: row.seq.to_string(),
                source: row.source,
                reason: row.reason,
                refused_at: row.refused_at,
            }));

            let (mut total, mut hydrated) = (0, 0);
            self.scan(&collection.id, |item| {
                total += 1;
                hydrated += usize::from(item.object.is_some());
                Ok(ControlFlow::Continue(()))
            })?;
            let queued = self.client.reader.count_pending_creates(&collection.id)?;

            calendars.push(PimdirCalendarStatus {
                name: if collection.name.is_empty() {
                    collection.id.clone()
                } else {
                    collection.name
                },
                id: collection.id,
                total,
                hydrated,
                queued,
            });
        }

        calendars.sort_by(|a, b| a.name.cmp(&b.name));

        let identity = self.store_identity()?;
        let parked = self
            .client
            .reader
            .parked_actions()?
            .into_iter()
            .filter(|parked| {
                calendars
                    .iter()
                    .any(|calendar| calendar.id == parked.collection)
            })
            .map(|parked| PimdirParkedStatus {
                row: parked.id,
                handle: crate::pimdir::queue::handle(&identity, parked.id),
                collection: parked.collection,
                kind: parked.action,
                error: parked.error,
            })
            .collect();

        Ok(PimdirStatusOutput {
            account: self.client.account.clone(),
            accounts,
            calendars,
            parked,
            refused,
        })
    }

    /// The store's calendar collections.
    ///
    /// Narrowed to the account read, configured or derived (pimdir
    /// STORAGE §9.2), every collection when the store groups none.
    pub(crate) fn calendar_collections(&self) -> Result<Vec<PimdirCollection>> {
        let collections = match self.client.account.as_deref() {
            Some(account) => self
                .client
                .reader
                .list_collections_by_account(Some(account))?,
            None => self.client.reader.list_collections()?,
        };

        Ok(collections
            .into_iter()
            .filter(|collection| media_type(&collection.kind) == CALENDAR_KIND)
            .collect())
    }

    /// Fails unless `calendar_id` names a calendar the store knows.
    ///
    /// The read seam answers an unknown collection with an empty page
    /// and the queue takes any name, so a typo in `-k` would read as an
    /// empty calendar and stage into one nothing ever applies.
    pub(crate) fn known_collection(&self, calendar_id: &str) -> Result<()> {
        let mut ids: Vec<String> = self
            .calendar_collections()?
            .into_iter()
            .map(|collection| collection.id)
            .collect();

        if ids.iter().any(|id| id == calendar_id) {
            return Ok(());
        }

        ids.sort();

        Err(calendar_not_found(format!(
            "Calendar `{calendar_id}` not found; this account holds: {}",
            ids.join(", "),
        )))
    }

    /// Walks every live item of a collection by keyset paging, each
    /// with its summary joined, until `visit` breaks.
    ///
    /// In the order the store maintains for calendars, start ascending.
    /// One batch of rows is held at a time, never the collection.
    fn scan(
        &self,
        calendar_id: &str,
        mut visit: impl FnMut(PimdirItem) -> Result<ControlFlow<()>>,
    ) -> Result<()> {
        let mut cursor: Option<(String, i64)> = None;

        loop {
            let batch = self.client.reader.list_summaries(
                calendar_id,
                cursor.as_ref().map(|(key, seq)| (key.as_str(), *seq)),
                SCAN_BATCH,
            )?;
            let count = batch.len();

            if let Some(last) = batch.last() {
                cursor = Some((last.sort_key.clone(), last.seq));
            }

            for item in batch {
                if visit(item)?.is_break() {
                    return Ok(());
                }
            }

            if count < SCAN_BATCH {
                return Ok(());
            }
        }
    }

    /// The public ids of the items a pending `update` of `calendar_id`
    /// repoints, or `None` when that calendar's queue does not decode.
    ///
    /// The reader folds the new body in, while the summary beside the
    /// row is still the one the owner derived from the old body, so it
    /// cannot rule such an item out. A queue that does not decode hides
    /// which items those are, so its calendar has none ruled out, with a
    /// warning, rather than its listing failing: `pimdir queue list`
    /// names the row.
    fn restaged(&self, calendar_id: &str) -> Result<Option<Vec<i64>>> {
        let pending = match self.client.reader.pending_actions(calendar_id) {
            Ok(pending) => pending,
            Err(err @ PimdirError::Action(_)) => {
                warn!("Read every body of calendar `{calendar_id}`, its queue undecoded: {err}");
                return Ok(None);
            }
            Err(err) => return Err(store_error(err, "Read the pimdir queue")),
        };

        let restaged = pending
            .into_iter()
            .filter_map(|pending| match pending.action {
                PimdirAction::Update { seq, .. } => Some(seq),
                _ => None,
            })
            .collect();

        Ok(Some(restaged))
    }

    /// The stored item behind a public item id, or a clear miss.
    ///
    /// Anything non-numeric is the internal link id or a mistyped
    /// value, and saying so beats a lookup that silently finds nothing.
    /// Both are `item-not-found`, an id naming no item.
    pub(crate) fn item(&self, calendar_id: &str, item_id: &str) -> Result<PimdirItem> {
        let seq = item_id
            .parse::<i64>()
            .map_err(|_| error::invalid_item_id(item_id, "a number"))?;

        self.client
            .reader
            .get_item(calendar_id, seq)?
            .ok_or_else(|| {
                item_not_found(format!(
                    "Item `{item_id}` not found in calendar `{calendar_id}`"
                ))
            })
    }

    /// Projects a stored item onto the shared type, body when local.
    ///
    /// Empty contents mean two things and only one is ordinary: the
    /// item is not hydrated, never fetched or its blob gone or failing its
    /// name (pimdir GUIDE §14), or its object's body cannot be read, an
    /// inconsistent store, whose `item-unreadable` entry comes beside it
    /// since the row renders alike. Either way it has no version.
    fn item_from(
        &self,
        calendar_id: &str,
        stored: &PimdirItem,
    ) -> (CalendarItem, Option<ItemError>) {
        let id = stored.seq.to_string();

        let read = match &stored.object {
            Some(hash) => match self.client.blobs.get(hash) {
                Ok(contents) => Ok(contents),
                Err(err) => Err(anyhow::Error::from(err).context(format!(
                    "Cannot read the body of item `{id}` in calendar `{calendar_id}`"
                ))),
            },
            None => Ok(None),
        };

        let (contents, error) = match read {
            Ok(contents) => (contents, None),
            Err(err) => (None, Some(error::unreadable(Some(&id), err))),
        };
        let etag = match contents {
            Some(_) => stored.object.as_ref().map(|hash| hash.0.clone()),
            None => None,
        };

        let mut item = CalendarItem {
            id,
            calendar_id: calendar_id.to_owned(),
            etag,
            contents: contents.unwrap_or_default(),
            stand_in: None,
            windowed: None,
        };

        if item.contents.is_empty()
            && error.is_none()
            && let Some(PimdirSummary::Event(_)) = &stored.summary
        {
            let event = crate::pimdir::index::event_hit(&item, Some(stored), String::new());
            item.stand_in = Some(Box::new(event));
        }

        (item, error)
    }

    /// Queues the account's reply to an invitation, the `calendar-reply`
    /// intent (pimdir STORAGE Annex B.2): the performer sends it to the
    /// organizer with `comment`, and the new `PARTSTAT` arrives with the
    /// next sync.
    ///
    /// `recurrence_id` limits it to one occurrence of a series, spelled
    /// as the item's `RECURRENCE-ID` would be (the series' `DTSTART`
    /// form), and needs a performer declaring `calendar.reply.occurrence`.
    /// `source` picks the performer for this action alone, among several
    /// able to. Returns the queue row and the performer named in it.
    pub fn reply(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        partstat: &str,
        recurrence_id: Option<&str>,
        comment: Option<&str>,
        source: Option<&str>,
    ) -> Result<PimdirQueued> {
        self.known_collection(calendar_id)?;

        let partstat = PimdirPartstat::parse(partstat)
            .ok_or_else(|| anyhow!("Invalid PARTSTAT `{partstat}` for a reply"))?;

        let stored = self.item(calendar_id, item_id)?;
        if let Some(contents) = self.body(calendar_id, &stored)? {
            if !names(&contents, "ORGANIZER") {
                return Err(invitation_missing(format!(
                    "Event `{item_id}` names no ORGANIZER, so it is not an invitation \
                     and there is nobody to reply to"
                )));
            }
            // NOTE: the account answers as one of the attendees, and an
            // event naming none is its own, or nobody's invitation.
            if !names(&contents, "ATTENDEE") {
                return Err(invitation_missing(format!(
                    "Event `{item_id}` names no ATTENDEE, so nobody is invited to it \
                     and there is no answer to give"
                )));
            }
            check_occurrence(calendar_id, &stored, &contents, recurrence_id)?;
        }

        self.intent(calendar_id, CALENDAR_REPLY, source, |source| {
            PimdirInvitation {
                source,
                item: PimdirIntentItem::Seq(stored.seq),
                partstat: Some(partstat),
                comment: comment.map(ToOwned::to_owned),
                recurrence_id: recurrence_id.map(ToOwned::to_owned),
            }
        })
    }

    /// Queues the cancellation of an event the account organises, the
    /// `calendar-cancel` intent (pimdir STORAGE Annex B.2): the performer
    /// notifies the attendees with `comment`, and the removal arrives
    /// with the next sync.
    ///
    /// `recurrence_id` limits it to one occurrence, as for
    /// [`reply`](Self::reply), and needs `calendar.cancel.occurrence`. An
    /// event inviting nobody has nobody to notify: `event delete` removes
    /// it.
    pub fn cancel(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        recurrence_id: Option<&str>,
        comment: Option<&str>,
        source: Option<&str>,
    ) -> Result<PimdirQueued> {
        self.known_collection(calendar_id)?;

        let stored = self.item(calendar_id, item_id)?;
        if let Some(contents) = self.body(calendar_id, &stored)? {
            if !names(&contents, "ATTENDEE") {
                return Err(invitation_missing(format!(
                    "Event `{item_id}` names no ATTENDEE, so there is nobody to notify; \
                     remove it with `event delete` instead"
                )));
            }
            check_occurrence(calendar_id, &stored, &contents, recurrence_id)?;
        }

        self.intent(calendar_id, CALENDAR_CANCEL, source, |source| {
            PimdirInvitation {
                source,
                item: PimdirIntentItem::Seq(stored.seq),
                partstat: None,
                comment: comment.map(ToOwned::to_owned),
                recurrence_id: recurrence_id.map(ToOwned::to_owned),
            }
        })
    }

    /// Appends an invitation intent, anchored on its collection (pimdir
    /// STORAGE §15.6), naming its performer.
    ///
    /// `invitation` builds the payload around the performer, which every
    /// intent names (Annex B.2): `source` when the user chose one, else
    /// the candidate the store resolves. A store whose sources declare
    /// nothing predates capabilities and resolves none, so the performer
    /// there is `source`, else the one source syncing the calendar.
    fn intent(
        &mut self,
        calendar_id: &str,
        capability: &str,
        source: Option<&str>,
        invitation: impl FnOnce(String) -> PimdirInvitation,
    ) -> Result<PimdirQueued> {
        let mut producer = self.client.producer()?;

        let syncing = producer.capabilities(calendar_id).map_err(|err| {
            store_error(
                err,
                format_args!("Read the capabilities of `{calendar_id}`"),
            )
        })?;
        let declared = syncing.iter().any(|source| source.declared.is_some());

        let performer = match (declared, source) {
            (false, Some(chosen)) => {
                if !syncing.iter().any(|source| source.source == chosen) {
                    let sources: Vec<&str> = syncing
                        .iter()
                        .map(|source| source.source.as_str())
                        .collect();
                    return Err(CodedError::new(
                        ErrorCode::NoPerformer,
                        format!(
                            "Source `{chosen}` does not sync calendar `{calendar_id}`, so it \
                             cannot perform {capability} there; choose one of: {}",
                            sources.join(", ")
                        ),
                    )
                    .into());
                }
                chosen.to_owned()
            }
            (false, None) => match syncing.as_slice() {
                [only] => only.source.clone(),
                [] => {
                    return Err(CodedError::new(
                        ErrorCode::NoPerformer,
                        format!(
                            "No source syncs calendar `{calendar_id}` to perform \
                             {capability}: name one with --source"
                        ),
                    )
                    .into());
                }
                several => {
                    let sources: Vec<&str> = several
                        .iter()
                        .map(|source| source.source.as_str())
                        .collect();
                    return Err(CodedError::new(
                        ErrorCode::AmbiguousPerformer,
                        format!(
                            "Several sources sync calendar `{calendar_id}` ({}): choose the \
                             one to perform {capability} with --source",
                            sources.join(", "),
                        ),
                    )
                    .into());
                }
            },
            (true, chosen) => match producer.performer(calendar_id, capability, chosen) {
                Ok(source) => source,
                Err(PimdirError::Ambiguous { candidates, .. }) => {
                    return Err(CodedError::new(
                        ErrorCode::AmbiguousPerformer,
                        format!(
                            "Several sources can perform {capability} for this account \
                             ({}): choose one with --source",
                            candidates.join(", "),
                        ),
                    )
                    .into());
                }
                Err(err) => {
                    return Err(store_error(err, format!("Find who performs {capability}")));
                }
            },
        };

        let invitation = invitation(performer);
        let PimdirIntentItem::Seq(seq) = invitation.item else {
            bail!("An invitation intent of calendula names a stored item");
        };

        let action = invitation
            .to_action()
            .map_err(|err| store_error(err.into(), format!("Queue {capability}")))?;

        let refuse = |err: PimdirError| match err {
            PimdirError::Unsupported(refusal)
                if refusal.capability == CALENDAR_REPLY_OCCURRENCE
                    || refusal.capability == CALENDAR_CANCEL_OCCURRENCE =>
            {
                let who = match declared {
                    false => "no source of this store declares it".to_owned(),
                    true => format!("source {} does not support it", refusal.source),
                };
                anyhow::Error::from(CodedError::new(
                    ErrorCode::OccurrenceUnsupported,
                    format!(
                        "{OCCURRENCE_UNSUPPORTED}: {capability} needs {}, and {who}; omit \
                         --recurrence-id to act on the whole series",
                        refusal.capability,
                    ),
                ))
            }
            err => store_error(err, format!("Queue {capability}")),
        };

        let enqueued = producer
            .enqueue(calendar_id, &action, None)
            .map_err(refuse)?;

        for partial in &enqueued.partial {
            warn!("{partial}");
        }

        Ok(PimdirQueued {
            handle: enqueued.handle,
            seq,
            source: invitation.source,
        })
    }

    /// The body the blob tree holds for `stored`, `None` when it is not
    /// local: never fetched, or its blob gone (pimdir STORAGE §7).
    fn local_body(&self, stored: &PimdirItem) -> Result<Option<Vec<u8>>> {
        match &stored.object {
            Some(hash) => Ok(self.client.blobs.get(hash)?),
            None => Ok(None),
        }
    }

    /// An item's local body, `None` when the sync has not fetched it.
    ///
    /// An item naming an object whose blob is absent, or fails its name,
    /// fails `body-pending`: not hydrated (pimdir GUIDE §14), and a write
    /// or an intent checked against no body would pass unchecked.
    fn body(&self, calendar_id: &str, stored: &PimdirItem) -> Result<Option<Vec<u8>>> {
        let Some(hash) = &stored.object else {
            return Ok(None);
        };

        match self.client.blobs.get(hash)? {
            Some(contents) => Ok(Some(contents)),
            None => Err(body_pending(&stored.seq.to_string(), calendar_id)),
        }
    }
}

/// The failure of a read of an item whose body is not local, coded
/// `body-pending`: the cue to sync, as Himalaya reads such a message.
fn body_pending(item_id: &str, calendar_id: &str) -> anyhow::Error {
    CodedError::new(
        ErrorCode::BodyPending,
        format!(
            "Item `{item_id}` in calendar `{calendar_id}` is not downloaded yet \
             (body not fetched); run a sync to hydrate it"
        ),
    )
    .into()
}

/// Stages `body` and appends the action `action` builds from its hash
/// through `producer` as one step, past the store's gate and, for an
/// `add`, its duplicate check (pimdir STORAGE §15.6, §15.1): a write the
/// store refuses leaves no body in the tree.
///
/// Logs the partial supports the gate passed the action with, read with
/// the body in hand, those the body asks for included, and answers the
/// row's receipt.
///
/// A [`ItemOrigin::Foreign`] body is queued without its `X-PIMDIR-`
/// properties (pimdir STORAGE §18), a [`ItemOrigin::User`] one whole.
fn enqueue_with_body(
    producer: &mut PimdirProducer,
    calendar_id: &str,
    body: &[u8],
    origin: ItemOrigin,
    action: impl FnOnce(PimdirHash) -> PimdirAction,
) -> Result<PimdirEnqueued> {
    let enqueued = match origin {
        ItemOrigin::User => producer.enqueue_user_body(calendar_id, body, action),
        ItemOrigin::Foreign => producer.enqueue_with_body(calendar_id, body, action),
    }
    .map_err(staging_error)?;

    for partial in &enqueued.partial {
        warn!("{partial}");
    }
    Ok(enqueued)
}

/// What a write the store refuses to stage reads as, naming the way out
/// where there is one.
fn staging_error(err: PimdirError) -> anyhow::Error {
    let staged = |code: ErrorCode, message: String| -> anyhow::Error {
        anyhow::Error::new(CodedError::new(code, message)).context("Stage the pimdir action")
    };
    let unsupported = |message: String| staged(ErrorCode::Unsupported, message);

    match err {
        PimdirError::Unsupported(refusal) if refusal.capability == CALENDAR_SCHEDULING => {
            unsupported(format!(
                "{refusal}; to write it without notifying \
                 anyone, first mark its ORGANIZER and ATTENDEE with \
                 SCHEDULE-AGENT=NONE"
            ))
        }
        PimdirError::Unsupported(refusal) if refusal.capability == CALENDAR_ONLINE_MEETING => {
            let who = match refusal.source.is_empty() {
                true => "no source of this store declares it".to_owned(),
                false => format!("source {} does not support it", refusal.source),
            };
            unsupported(format!(
                "An online meeting (X-PIMDIR-ONLINE-MEETING) needs \
                 {}, and {who}; drop --online-meeting to write the event without one",
                refusal.capability,
            ))
        }
        PimdirError::Unsupported(refusal) if refusal.capability == CALENDAR_COMPONENT_VTODO => {
            let why = match &refusal.detail {
                Some(detail) => format!(" ({detail})"),
                None => String::new(),
            };
            unsupported(format!(
                "This calendar takes no to-do, source {} \
                 states {} none{why}; store it in another calendar with -k/--calendar",
                refusal.source, refusal.capability,
            ))
        }
        PimdirError::Duplicate {
            collection,
            link_id,
            existing,
        } => {
            let message = match existing {
                PimdirDuplicate::Live(seq) => format!(
                    "Calendar `{collection}` already holds \
                     `{link_id}` as item {seq}; update that item instead"
                ),
                PimdirDuplicate::Pending(row) => format!(
                    "Calendar `{collection}` already has a \
                     create of `{link_id}` queued (row {row}) for the next sync"
                ),
            };
            staged(ErrorCode::ItemDuplicate, message)
        }
        err => store_error(err, "Stage the pimdir action"),
    }
}

/// How the refusal of a reply or a cancel naming one occurrence starts
/// when its performer cannot act on one alone, a prefix callers may match
/// on.
pub const OCCURRENCE_UNSUPPORTED: &str = "Occurrence not supported";

/// Fails unless `contents` holds the occurrence `recurrence_id` names,
/// spelled exactly as an expanded listing reports its `recurrenceId`.
///
/// The spelling is the payload's (pimdir STORAGE Annex B.2): the
/// series' `DTSTART` form, which a performer matches verbatim, so a
/// value naming the right instant in another form is refused too.
fn check_occurrence(
    calendar_id: &str,
    stored: &PimdirItem,
    contents: &[u8],
    recurrence_id: Option<&str>,
) -> Result<()> {
    let Some(recurrence_id) = recurrence_id else {
        return Ok(());
    };

    let item = CalendarItem {
        id: stored.seq.to_string(),
        calendar_id: calendar_id.to_owned(),
        etag: None,
        contents: contents.to_vec(),
        stand_in: None,
        windowed: None,
    };

    match Event::occurrence(&item, recurrence_id) {
        Some(event) if event.recurrence_id.as_deref() == Some(recurrence_id) => Ok(()),
        Some(event) => bail!(
            "Event `{}` spells that occurrence `{}`, not `{recurrence_id}`",
            stored.seq,
            event.recurrence_id.unwrap_or_default(),
        ),
        None => bail!("Event `{}` has no occurrence `{recurrence_id}`", stored.seq),
    }
}

/// How the refusal of a write whose `--if-match` names a version the
/// item no longer has starts, a prefix callers may match on.
pub const PRECONDITION_FAILED: &str = "Precondition failed";

/// Fails unless `if_match` names the item's current version.
///
/// The version is the store's hash of the item's body, the pending
/// queue folded in: the `etag` a read or a listing reports. An item
/// whose body is not local has no version to match. Surrounding double
/// quotes are dropped, so an HTTP-quoted tag matches as well.
fn check_version(
    held: Option<&PimdirHash>,
    calendar_id: &str,
    item_id: &str,
    if_match: Option<&str>,
) -> Result<()> {
    let Some(expected) = if_match else {
        return Ok(());
    };
    let expected = expected.trim().trim_matches('"');
    let current = held.map(|hash| hash.0.as_str());

    if current == Some(expected) {
        return Ok(());
    }

    Err(CodedError::new(
        ErrorCode::PreconditionFailed,
        format!(
            "{PRECONDITION_FAILED}: item `{item_id}` in calendar `{calendar_id}` is at \
             version `{}`, not `{expected}`; read it again",
            current.unwrap_or("none (body not fetched)"),
        ),
    )
    .into())
}

/// Whether the stored summary proves a local item matches neither `kind`
/// nor `uid`, so a listing need not read its body.
///
/// Only a negative answer is drawn, and only from what the format makes
/// true of every resource: one component type, filed in that type's
/// table (pimdir STORAGE Annex A.3), and one `UID` shared by every
/// component (RFC 4791 4.1). A date never rules an item out: an
/// override moves an occurrence anywhere and the summary describes the
/// master alone, so the window is answered from the body.
///
/// An item with no local body is answered from its summary anyway,
/// without a read, and one a pending `update` repoints (`restaged`)
/// carries a summary of its old body, so neither is ruled out here.
fn ruled_out(
    stored: &PimdirItem,
    kind: Option<IcalComponentKind>,
    uid: Option<&str>,
    restaged: &[i64],
) -> bool {
    if stored.object.is_none() || restaged.contains(&stored.seq) {
        return false;
    }

    let (table, stated) = match &stored.summary {
        Some(PimdirSummary::Event(event)) => (IcalComponentKind::VEvent, event.uid.as_deref()),
        Some(PimdirSummary::Task(task)) => (IcalComponentKind::VTodo, task.uid.as_deref()),
        Some(PimdirSummary::Journal(journal)) => {
            (IcalComponentKind::VJournal, journal.uid.as_deref())
        }
        _ => return false,
    };

    if kind.is_some_and(|kind| kind != table) {
        return true;
    }

    // NOTE: the summary keeps the `UID` verbatim where a projection
    // decodes it, so a value holding an escape or padding is left to
    // the body rather than compared.
    match (uid, stated) {
        (Some(uid), Some(stated)) => !stated.contains('\\') && stated.trim() != uid.trim(),
        _ => false,
    }
}

/// The window a streamed listing keeps: `skip` matches passed over, then
/// at most `take` kept, a `None` take keeping every match, as the shared
/// `window` cuts a whole list.
struct Window {
    /// Matches still to pass over before the window starts.
    skip: usize,
    /// The window size, `None` for no cap.
    take: Option<usize>,
    /// The matches kept, and the error of each kept one that could not be
    /// read.
    list: CalendarItemList,
}

impl Window {
    fn new(skip: usize, take: Option<usize>) -> Self {
        Self {
            skip,
            take,
            list: CalendarItemList::default(),
        }
    }

    /// Whether the window holds nothing whatever the collection holds.
    fn is_closed(&self) -> bool {
        self.take == Some(0)
    }

    /// Passes over one match when the window has not started yet.
    fn skips(&mut self) -> bool {
        match self.skip {
            0 => false,
            _ => {
                self.skip -= 1;
                true
            }
        }
    }

    /// Keeps one match with the error of its read, breaking once the
    /// window is full.
    fn keep(&mut self, item: CalendarItem, error: Option<ItemError>) -> ControlFlow<()> {
        self.list.items.push(item);
        self.list.errors.extend(error);

        match self.take {
            Some(take) if self.list.items.len() >= take => ControlFlow::Break(()),
            _ => ControlFlow::Continue(()),
        }
    }

    fn into_list(self) -> CalendarItemList {
        self.list
    }
}

/// Whether a stored item is of `kind`.
///
/// A body the sync has not downloaded still answers: the summary table
/// the store filed it in names the component a reader renders the
/// resource as (pimdir STORAGE Annex A.3 to A.5), so an
/// availability-aware listing stays one.
fn is_kind(item: &CalendarItem, stored: &PimdirItem, kind: IcalComponentKind) -> bool {
    if !item.contents.is_empty() {
        return may_hold_kind(&item.contents, kind);
    }

    match &stored.summary {
        Some(PimdirSummary::Event(_)) => kind == IcalComponentKind::VEvent,
        Some(PimdirSummary::Task(_)) => kind == IcalComponentKind::VTodo,
        Some(PimdirSummary::Journal(_)) => kind == IcalComponentKind::VJournal,
        _ => false,
    }
}

/// Whether an item falls inside `range`.
///
/// A hydrated item is answered from its own bytes, which is exact: an
/// event starting in it, or an occurrence of a series overlapping it.
/// One with no local body falls back to the start its summary carries,
/// the whole point of a summary beside the pointer, and a series the
/// summary says recurs passes from that start to its `UNTIL`.
///
/// The bytes are parsed once: the occurrences they hold in `range` stay
/// on the item ([`CalendarItem::windowed`]), so the listing projecting
/// them reads them back rather than parsing the item again. Bytes that
/// do not parse keep the item, for the listing to name.
fn in_range(item: &mut CalendarItem, stored: &PimdirItem, range: &CalendarTimeRange) -> bool {
    if !item.contents.is_empty() {
        let Ok((inside, events)) = expand::windowed(item, range) else {
            return true;
        };

        if inside {
            item.windowed = Some(Box::new(CalendarItemWindowed {
                range: range.clone(),
                events,
            }));
        }

        return inside;
    }

    if let Some(PimdirSummary::Event(event)) = &stored.summary
        && event.recurring == Some(true)
        && let Some(start) = &event.dtstart
    {
        let day = |stamp: &str| stamp.get(..8).unwrap_or(stamp).to_owned();
        let started = range
            .end
            .as_deref()
            .is_none_or(|end| day(&start.value) < day(end));
        let running = match (event.until.as_deref(), range.start.as_deref()) {
            (Some(until), Some(from)) => day(until) >= day(from),
            _ => true,
        };
        return started && running;
    }

    start_of(stored).is_some_and(|start| range.contains(&start.value))
}

/// The time a summary answers a date question with: `DTSTART`, then
/// `DUE` for a to-do carrying no start. Verbatim, so its leading day
/// compares as a parsed body's `DTSTART` does.
fn start_of(stored: &PimdirItem) -> Option<&PimdirTime> {
    match stored.summary.as_ref()? {
        PimdirSummary::Event(event) => event.dtstart.as_ref(),
        PimdirSummary::Task(task) => task.dtstart.as_ref().or(task.due.as_ref()),
        PimdirSummary::Journal(journal) => journal.dtstart.as_ref(),
        PimdirSummary::Mail(_) | PimdirSummary::Contact(_) | PimdirSummary::File(_) => None,
    }
}

/// What queueing an intent wrote.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PimdirQueued {
    /// The queue row's handle, its id under the store's identity (pimdir
    /// STORAGE §4.5), which `pimdir queue show` and `cancel` take.
    pub handle: PimdirActionHandle,
    /// The public id of the item the intent addresses.
    pub seq: i64,
    /// The source named to perform it.
    pub source: String,
}

/// A refusal under `invitation-missing`: the event a reply or a cancel
/// names invites nobody it could address.
fn invitation_missing(message: String) -> anyhow::Error {
    CodedError::new(ErrorCode::InvitationMissing, message).into()
}

/// Whether a VEVENT of `contents` carries the property `name`.
///
/// A body that does not parse answers yes: refusing an intent is the
/// owner's call when the producer cannot read what it addresses.
fn names(contents: &[u8], name: &str) -> bool {
    let Ok(cst) = ical::tree::cst::IcalCst::parse(contents) else {
        return true;
    };
    let ical = cst.decode();

    ical.components
        .iter()
        .filter(|component| {
            matches!(
                component.name,
                ical::component::IcalComponentName::Kind(IcalComponentKind::VEvent)
            )
        })
        .any(|component| {
            component
                .props
                .iter()
                .any(|prop| prop.name.eq_ignore_ascii_case(name))
        })
}

/// The message a collection verb refuses with, naming the way out.
fn unsupported(verb: &str) -> String {
    format!(
        "pimdir cannot {verb} a calendar: the store is an offline cache, and its collections \
         come from the sync engine that fills it. Run the operation against the account the \
         store syncs, then sync again."
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        backend::Backend,
        config::{AccountConfig, Config},
        shared::{client::CalendarClient, ical::IcalFamily},
    };
    use io_pimdir::{
        placement::{PimdirLevel, PimdirLinkId},
        summary::calendar::{PimdirEventSummary, PimdirJournalSummary, PimdirTaskSummary},
    };

    use super::*;

    /// The calendar the server names the default carries the mark, read
    /// from the role the sync engine recorded (pimdir STORAGE §14).
    #[test]
    fn the_default_calendar_is_the_one_its_store_marks() {
        let dir = crate::pimdir::client::store_dir();
        {
            let store = io_pimdir::client::PimdirStore::open(dir.path())
                .unwrap()
                .for_account("work");
            for id in ["caldav/default", "caldav/team"] {
                store.ensure_collection(id, CALENDAR_KIND).unwrap();
            }
            store
                .set_collection_role("caldav/default", Some("default"))
                .unwrap();
        }

        let mut backend = PimdirBackend::new(crate::config::PimdirConfig {
            root: dir.path().to_path_buf(),
            account: None,
        })
        .unwrap();
        let defaults: Vec<(String, bool)> = backend
            .list_calendars()
            .unwrap()
            .into_iter()
            .map(|calendar| (calendar.id, calendar.default))
            .collect();
        assert_eq!(
            defaults,
            [
                ("caldav/default".into(), true),
                ("caldav/team".into(), false)
            ]
        );
    }

    /// A client reads one account: the one a store groups everything
    /// under when `pimdir.account` is unset, and never several merged.
    #[test]
    fn an_unset_account_reads_the_only_one_and_refuses_several() {
        let dir = crate::pimdir::client::store_dir();
        let open = || {
            PimdirBackend::new(crate::config::PimdirConfig {
                root: dir.path().to_path_buf(),
                account: None,
            })
        };

        io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_account("work")
            .ensure_collection("caldav/work", CALENDAR_KIND)
            .unwrap();
        let mut backend = open().unwrap();
        assert_eq!(backend.status().unwrap().account.as_deref(), Some("work"));
        drop(backend);

        io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_account("home")
            .ensure_collection("caldav/home", CALENDAR_KIND)
            .unwrap();
        let err = open().err().unwrap().to_string();
        assert!(err.contains("home, work"), "{err}");
        assert!(err.contains("pimdir.account"), "{err}");
    }

    /// An account holding no calendar makes no store ambiguous, as
    /// himalaya counts accounts holding mail; several holding calendars
    /// do, coded.
    #[test]
    fn an_account_holding_no_calendar_is_not_counted() {
        use pimalaya_cli::error::code_of;

        let dir = crate::pimdir::client::store_dir();
        let open = || {
            PimdirBackend::new(crate::config::PimdirConfig {
                root: dir.path().to_path_buf(),
                account: None,
            })
        };
        for (account, collection, kind) in [
            ("work", "caldav/work", CALENDAR_KIND),
            ("cards", "carddav/book", "text/vcard"),
        ] {
            io_pimdir::client::PimdirStore::open(dir.path())
                .unwrap()
                .for_account(account)
                .ensure_collection(collection, kind)
                .unwrap();
        }
        let mut backend = open().unwrap();
        assert_eq!(backend.status().unwrap().account.as_deref(), Some("work"));
        drop(backend);

        io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_account("home")
            .ensure_collection("caldav/home", CALENDAR_KIND)
            .unwrap();
        let err = open().err().unwrap();
        assert_eq!(
            code_of(&err),
            Some(ErrorCode::StoreAccountAmbiguous),
            "{err:#}"
        );
        assert!(
            err.to_string()
                .contains("accounts of calendars (home, work)"),
            "{err:#}"
        );
    }

    /// A collection declaring no kind is no calendar: its account makes no
    /// store ambiguous, and no listing shows it.
    #[test]
    fn a_collection_of_no_kind_is_no_calendar() {
        let dir = crate::pimdir::client::store_dir();
        let open = |account: Option<&str>| {
            PimdirBackend::new(crate::config::PimdirConfig {
                root: dir.path().to_path_buf(),
                account: account.map(Into::into),
            })
        };
        for (account, collection, kind) in [
            ("work", "caldav/work", CALENDAR_KIND),
            ("old", "legacy", ""),
        ] {
            io_pimdir::client::PimdirStore::open(dir.path())
                .unwrap()
                .for_account(account)
                .ensure_collection(collection, kind)
                .unwrap();
        }

        let mut backend = open(None).unwrap();
        assert_eq!(backend.status().unwrap().account.as_deref(), Some("work"));
        drop(backend);

        let mut backend = open(Some("old")).unwrap();
        assert!(backend.list_calendars().unwrap().is_empty());
    }

    /// A `pimdir.account` is taken on a store holding no collection yet,
    /// a first sync not having run.
    #[test]
    fn a_configured_account_reads_an_empty_store() {
        let dir = crate::pimdir::client::store_dir();
        io_pimdir::client::PimdirStore::open(dir.path()).unwrap();

        let mut backend = PimdirBackend::new(crate::config::PimdirConfig {
            root: dir.path().to_path_buf(),
            account: Some("work".into()),
        })
        .unwrap();

        assert_eq!(backend.status().unwrap().account.as_deref(), Some("work"));
        assert!(backend.list_calendars().unwrap().is_empty());
    }

    /// CAL-14: a `pimdir.account` the store holds no collection of is
    /// refused, naming those it holds, rather than read as empty.
    #[test]
    fn a_configured_account_the_store_lacks_is_refused() {
        let dir = crate::pimdir::client::store_dir();
        io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_account("work")
            .ensure_collection("caldav/work", CALENDAR_KIND)
            .unwrap();

        let err = PimdirBackend::new(crate::config::PimdirConfig {
            root: dir.path().to_path_buf(),
            account: Some("wrok".into()),
        })
        .err()
        .unwrap();

        assert_eq!(
            pimalaya_cli::error::code_of(&err),
            Some(ErrorCode::StoreAccountUnknown)
        );
        let err = err.to_string();
        assert!(err.contains("no account `wrok` (work)"), "{err}");
    }

    /// A write the owner parks never lands, so status shows it by its
    /// queue row, for the account read alone.
    #[test]
    fn status_shows_the_parked_actions_of_the_account_read() {
        let dir = crate::pimdir::client::store_dir();
        for (account, collection) in [("work", "caldav/work"), ("home", "caldav/home")] {
            io_pimdir::client::PimdirStore::open(dir.path())
                .unwrap()
                .for_account(account)
                .ensure_collection(collection, CALENDAR_KIND)
                .unwrap();
            crate::pimdir::client::sync_from(dir.path(), Some(account), collection);
        }
        let open = |account: &str| {
            PimdirBackend::new(crate::config::PimdirConfig {
                root: dir.path().to_path_buf(),
                account: Some(account.into()),
            })
            .unwrap()
        };

        // NOTE: the second add of one `UID` is the one the owner parks,
        // the first having made it live. calendula refuses it before
        // queueing, so the producer stages it bare.
        for (account, collection) in [("work", "caldav/work"), ("home", "caldav/home")] {
            let mut backend = open(account);
            backend
                .create_item(collection, ALONE.into(), ItemOrigin::User)
                .unwrap();

            let mut producer = backend.client.producer().unwrap();
            let object = producer.stage_body(ALONE.as_bytes()).unwrap();
            let add = PimdirAction::Add {
                link_id: Some(derive(ALONE.as_bytes()).link_id),
                flags: PimdirFlags::default(),
                object: Some(object.hash.clone()),
                supersedes: None,
            };
            producer.enqueue(collection, &add, Some(&object)).unwrap();
        }
        let mut owner = io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_source("caldav");
        assert_eq!(owner.drain().unwrap().parked, 2);
        drop(owner);

        let status = open("work").status().unwrap();
        assert_eq!(status.parked.len(), 1, "{:?}", status.parked);
        let parked = &status.parked[0];
        assert_eq!(parked.collection, "caldav/work");
        assert_eq!(parked.kind, "add");
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["parked"][0]["kind"], "add", "{json}");
        assert!(json["parked"][0].get("action").is_none(), "{json}");
        assert!(
            parked.error.contains("alone@example.org"),
            "{}",
            parked.error
        );

        let text = status.to_string();
        let head = |line: &str| {
            line.split_whitespace()
                .eq(["ROW", "KIND", "CALENDAR", "ERROR"])
        };
        assert!(text.lines().any(head), "{text}");
        let handle: io_pimdir::codec::PimdirActionHandle = parked.handle.parse().unwrap();
        assert_eq!(handle.id, parked.row);
        assert!(handle.store.is_some(), "{handle:?}");
        assert_eq!(json["parked"][0]["handle"], parked.handle, "{json}");
        let row = parked.handle.clone();
        let words = |line: &str| line.split_whitespace().take(2).eq([row.as_str(), "add"]);
        assert!(text.lines().any(words), "{text}");
        assert!(text.contains(&parked.error), "{text}");
    }

    #[test]
    fn the_collection_refusal_points_at_the_sync() {
        let message = unsupported("create");
        assert!(message.contains("offline cache"));
        assert!(message.contains("sync"));
    }

    /// One calendar may hold two resources carrying one `UID` (pimdir
    /// STORAGE §9), which the store keys apart under their own public ids.
    ///
    /// What addresses an item here is that id, never the identity its
    /// body states, and the two copies need not be the same event.
    #[test]
    fn two_items_sharing_one_uid_list_under_their_own_public_ids() {
        let body = |summary: &str| {
            format!(
                "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:demo@example.org\r\n\
                 SUMMARY:{summary}\r\nDTSTART:20260814T090000Z\r\n\
                 END:VEVENT\r\nEND:VCALENDAR\r\n"
            )
            .into_bytes()
        };
        let item = |seq: &str, summary: &str| CalendarItem {
            id: seq.into(),
            calendar_id: "personal".into(),
            etag: None,
            contents: body(summary),
            stand_in: None,
            windowed: None,
        };

        let woonies = item("1", "Pre demo woonies");
        let minis = item("2", "Pre demo MINIS");

        // NOTE: the identity the two bodies state is one string, so it
        // names both of them and addresses neither.
        assert_eq!(
            derive(&woonies.contents).link_id,
            derive(&minis.contents).link_id
        );

        let events: Vec<Event> = [&woonies, &minis]
            .into_iter()
            .flat_map(Event::project)
            .collect();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].id, "1");
        assert_eq!(events[0].summary.as_deref(), Some("Pre demo woonies"));
        assert_eq!(events[1].id, "2");
        assert_eq!(events[1].summary.as_deref(), Some("Pre demo MINIS"));
    }

    #[test]
    fn an_undownloaded_item_is_windowed_from_its_summary() {
        let range = CalendarTimeRange {
            start: Some("20260801T000000Z".into()),
            end: Some("20260901T000000Z".into()),
        };
        let event = |start: &str| {
            stored(PimdirSummary::Event(PimdirEventSummary {
                dtstart: Some(time(start)),
                ..Default::default()
            }))
        };
        let task = |due: &str| {
            stored(PimdirSummary::Task(PimdirTaskSummary {
                due: Some(time(due)),
                ..Default::default()
            }))
        };
        let dateless = stored(PimdirSummary::Journal(PimdirJournalSummary::default()));

        assert!(in_range(
            &mut undownloaded(),
            &event("20260814T090000Z"),
            &range
        ));
        assert!(!in_range(
            &mut undownloaded(),
            &event("20260914T090000Z"),
            &range
        ));
        assert!(in_range(&mut undownloaded(), &task("20260814"), &range));
        assert!(!in_range(&mut undownloaded(), &dateless, &range));
    }

    #[test]
    fn an_undownloaded_item_answers_its_kind_from_its_summary() {
        let task = stored(PimdirSummary::Task(PimdirTaskSummary::default()));
        let bare = PimdirItem {
            summary: None,
            ..task.clone()
        };

        assert!(is_kind(&undownloaded(), &task, IcalComponentKind::VTodo));
        assert!(!is_kind(&undownloaded(), &task, IcalComponentKind::VEvent));
        assert!(!is_kind(&undownloaded(), &bare, IcalComponentKind::VTodo));
    }

    /// The shared view of an item the sync has not hydrated.
    fn undownloaded() -> CalendarItem {
        CalendarItem {
            id: "1".into(),
            calendar_id: "personal".into(),
            etag: None,
            contents: Vec::new(),
            stand_in: None,
            windowed: None,
        }
    }

    /// A stored row at the `Meta` tier carrying `summary`.
    fn stored(summary: PimdirSummary) -> PimdirItem {
        PimdirItem {
            seq: 1,
            link_id: PimdirLinkId("party@example.org".into()),
            flags: PimdirFlags::default(),
            sort_key: String::new(),
            object: None,
            level: PimdirLevel::Meta,
            summary: Some(summary),
            retention: None,
        }
    }

    /// A summary time carrying `value` verbatim.
    fn time(value: &str) -> PimdirTime {
        PimdirTime {
            value: value.into(),
            ..Default::default()
        }
    }

    /// An invitation the account received, with its organizer.
    const INVITATION: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//t//t//EN\r\n\
        BEGIN:VEVENT\r\nUID:invite@example.org\r\nDTSTAMP:20261001T080000Z\r\n\
        DTSTART:20261005T090000Z\r\nDTEND:20261005T100000Z\r\nSUMMARY:Review\r\n\
        ORGANIZER;CN=Alice:mailto:alice@example.org\r\n\
        ATTENDEE;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:bob@example.org\r\n\
        END:VEVENT\r\nEND:VCALENDAR\r\n";

    /// An event inviting nobody.
    const ALONE: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//t//t//EN\r\n\
        BEGIN:VEVENT\r\nUID:alone@example.org\r\nDTSTAMP:20261001T080000Z\r\n\
        DTSTART:20261005T090000Z\r\nSUMMARY:Focus\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

    /// A store holding one calendar, `cal`, synced by `caldav` and
    /// carrying `bodies` as items a sync applied, and the backend over it
    /// with their public ids.
    fn store(bodies: &[&str]) -> (crate::pimdir::client::StoreDir, PimdirBackend, Vec<String>) {
        let dir = crate::pimdir::client::store_dir();
        let store = io_pimdir::client::PimdirStore::open(dir.path()).unwrap();
        store.ensure_collection("cal", CALENDAR_KIND).unwrap();
        drop(store);
        crate::pimdir::client::sync_from(dir.path(), None, "cal");

        let config = || PimdirConfig {
            root: dir.path().to_path_buf(),
            account: None,
        };

        let mut backend = PimdirBackend::new(config()).unwrap();
        for body in bodies {
            backend
                .create_item("cal", body.as_bytes().to_vec(), ItemOrigin::User)
                .unwrap();
        }

        let mut owner = io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_source("caldav");
        owner.drain().unwrap();
        drop(owner);

        let mut backend = PimdirBackend::new(config()).unwrap();
        let ids = backend
            .list_items("cal", CalendarItemQuery::default())
            .unwrap()
            .items
            .into_iter()
            .map(|item| item.id)
            .collect();

        (dir, backend, ids)
    }

    /// The pending queue rows of `cal` as `(kind, payload)`.
    fn queued(backend: &PimdirBackend) -> Vec<(String, serde_json::Value)> {
        let producer = backend.client.producer().unwrap();
        producer
            .pending_actions("cal")
            .unwrap()
            .into_iter()
            .map(|pending| match pending.action {
                PimdirAction::Unknown { kind, payload, .. } => {
                    (kind, serde_json::from_str(&payload).unwrap())
                }
                action => panic!("expected an intent, got {action:?}"),
            })
            .collect()
    }

    #[test]
    fn a_reply_queues_one_calendar_reply_row_with_the_spec_payload() {
        let (_dir, mut backend, ids) = store(&[INVITATION]);

        let queued_row = backend
            .reply(
                "cal",
                &ids[0],
                "ACCEPTED",
                None,
                Some("See you there"),
                None,
            )
            .unwrap();

        let rows = queued(&backend);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "calendar-reply");
        assert_eq!(
            rows[0].1,
            serde_json::json!({
                "v": 1,
                "source": "caldav",
                "seq": queued_row.seq,
                "partstat": "ACCEPTED",
                "comment": "See you there",
            })
        );
        assert_eq!(queued_row.seq.to_string(), ids[0]);
        assert_eq!(queued_row.source, "caldav");
    }

    #[test]
    fn a_reply_without_comment_carries_no_comment_and_a_chosen_source() {
        let (_dir, mut backend, ids) = store(&[INVITATION]);

        backend
            .reply("cal", &ids[0], "DECLINED", None, None, Some("caldav"))
            .unwrap();

        let rows = queued(&backend);
        assert_eq!(
            rows[0].1,
            serde_json::json!({
                "v": 1,
                "seq": ids[0].parse::<i64>().unwrap(),
                "partstat": "DECLINED",
                "source": "caldav",
            })
        );
    }

    /// CAL-17: in a store whose sources declare nothing, a `--source`
    /// syncing nothing of the calendar is refused, a row naming it being
    /// one the sync engine would skip for ever.
    #[test]
    fn a_chosen_source_syncing_nothing_of_the_calendar_is_refused() {
        use pimalaya_cli::error::code_of;

        let (_dir, mut backend, ids) = store(&[INVITATION]);

        let reply = backend
            .reply("cal", &ids[0], "ACCEPTED", None, None, Some("bogus"))
            .unwrap_err();
        let cancel = backend
            .cancel("cal", &ids[0], None, None, Some("bogus"))
            .unwrap_err();

        for err in [reply, cancel] {
            assert_eq!(code_of(&err), Some(ErrorCode::NoPerformer), "{err:#}");
            assert!(err.to_string().contains("choose one of: caldav"), "{err:#}");
        }
        assert!(queued(&backend).is_empty());
    }

    #[test]
    fn a_cancel_queues_one_calendar_cancel_row() {
        let (_dir, mut backend, ids) = store(&[INVITATION]);

        backend
            .cancel("cal", &ids[0], None, Some("Moved to next week"), None)
            .unwrap();

        let rows = queued(&backend);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "calendar-cancel");
        assert_eq!(
            rows[0].1,
            serde_json::json!({
                "v": 1,
                "source": "caldav",
                "seq": ids[0].parse::<i64>().unwrap(),
                "comment": "Moved to next week",
            })
        );
    }

    #[test]
    fn an_event_with_nobody_to_answer_or_notify_queues_nothing() {
        use pimalaya_cli::error::code_of;

        let (_dir, mut backend, ids) = store(&[ALONE]);

        let reply = backend.reply("cal", &ids[0], "ACCEPTED", None, None, None);
        let reply = reply.unwrap_err();
        assert!(reply.to_string().contains("ORGANIZER"), "{reply}");
        assert_eq!(code_of(&reply), Some(ErrorCode::InvitationMissing));

        let cancel = backend.cancel("cal", &ids[0], None, None, None);
        let cancel = cancel.unwrap_err();
        assert!(cancel.to_string().contains("event delete"), "{cancel}");
        assert_eq!(code_of(&cancel), Some(ErrorCode::InvitationMissing));

        assert!(queued(&backend).is_empty());
    }

    /// An event naming its organizer and no attendee, as Graph writes the
    /// account's own, invites nobody to answer as: a reply is refused
    /// before it is queued, never parked by the server after a sync.
    #[test]
    fn a_reply_to_an_event_inviting_nobody_queues_nothing() {
        use pimalaya_cli::error::code_of;

        let own = ALONE.replace(
            "SUMMARY:Focus\r\n",
            "SUMMARY:Focus\r\nORGANIZER:mailto:me@example.org\r\n",
        );
        let (_dir, mut backend, ids) = store(&[&own]);

        let reply = backend.reply("cal", &ids[0], "ACCEPTED", None, None, None);
        let reply = reply.unwrap_err();
        assert!(reply.to_string().contains("names no ATTENDEE"), "{reply}");
        assert_eq!(code_of(&reply), Some(ErrorCode::InvitationMissing));

        assert!(queued(&backend).is_empty());
    }

    #[test]
    fn an_unknown_event_or_calendar_queues_nothing() {
        let (_dir, mut backend, _) = store(&[INVITATION]);

        assert!(
            backend
                .reply("cal", "999", "ACCEPTED", None, None, None)
                .is_err()
        );
        assert!(
            backend
                .reply("nope", "1", "ACCEPTED", None, None, None)
                .is_err()
        );
        assert!(queued(&backend).is_empty());
    }

    /// Every intent names its performer (pimdir STORAGE Annex B.2): in a
    /// store whose sources declare nothing, a calendar several sources
    /// sync has no performer until the user names one, and nothing is
    /// queued meanwhile.
    #[test]
    fn an_undeclared_calendar_several_sources_sync_needs_a_source() {
        use io_pimdir::{
            change::PimdirWriteOp,
            collection::{PimdirCheckpoint, PimdirCollectionId},
        };

        let (dir, mut backend, ids) = store(&[INVITATION]);
        let mut owner = io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_source("google");
        owner
            .write(vec![PimdirWriteOp::SetCheckpoint {
                collection: PimdirCollectionId("cal".into()),
                checkpoint: PimdirCheckpoint(b"1".to_vec()),
            }])
            .unwrap();
        drop(owner);

        let reply = backend.reply("cal", &ids[0], "ACCEPTED", None, None, None);
        let cancel = backend.cancel("cal", &ids[0], None, None, None);
        for err in [reply.unwrap_err(), cancel.unwrap_err()] {
            let err = err.to_string();
            assert!(err.contains("caldav, google"), "{err}");
            assert!(err.contains("--source"), "{err}");
        }
        assert!(queued(&backend).is_empty());

        let chosen = backend
            .reply("cal", &ids[0], "ACCEPTED", None, None, Some("google"))
            .unwrap();
        assert_eq!(chosen.source, "google");
        assert_eq!(queued(&backend)[0].1["source"], "google");
    }

    /// What a reply and a cancel queue reads back under the payload's own
    /// schema (pimdir STORAGE Annex B.2), which a performer reads strictly:
    /// a named performer, the item by its `seq` alone, a `partstat` of the
    /// three on a reply and none on a cancel.
    #[test]
    fn a_queued_reply_and_cancel_read_back_under_their_schema() {
        use io_pimdir::intent::{PimdirInvitation, PimdirPartstat as Partstat};

        let (_dir, mut backend, ids) = store(&[INVITATION]);
        let seq: i64 = ids[0].parse().unwrap();

        for (partstat, expected) in [
            ("ACCEPTED", Partstat::Accepted),
            ("TENTATIVE", Partstat::Tentative),
            ("DECLINED", Partstat::Declined),
        ] {
            backend
                .reply("cal", &ids[0], partstat, None, Some("Noted"), None)
                .unwrap();
            let (kind, payload) = queued(&backend).pop().unwrap();
            let read = PimdirInvitation::from_payload(&kind, &payload.to_string()).unwrap();
            assert_eq!(read.source, "caldav");
            assert_eq!(read.item, PimdirIntentItem::Seq(seq));
            assert_eq!(read.partstat, Some(expected));
        }

        backend.cancel("cal", &ids[0], None, None, None).unwrap();
        let (kind, payload) = queued(&backend).pop().unwrap();
        assert_eq!(payload.get("partstat"), None);
        let read = PimdirInvitation::from_payload(&kind, &payload.to_string()).unwrap();
        assert_eq!((read.source.as_str(), read.partstat), ("caldav", None));
    }

    /// A participation outside the three a reply sends is refused before
    /// anything is queued, as the payload's schema refuses it.
    #[test]
    fn a_reply_outside_the_three_participations_queues_nothing() {
        let (_dir, mut backend, ids) = store(&[INVITATION]);

        for partstat in ["NEEDS-ACTION", "DELEGATED", "accepted", ""] {
            let err = backend
                .reply("cal", &ids[0], partstat, None, None, None)
                .unwrap_err()
                .to_string();
            assert!(err.contains("PARTSTAT"), "{err}");
        }
        assert!(queued(&backend).is_empty());
    }

    /// A read and a listing report the one version, the body's hash,
    /// and a staged update moves it at once.
    #[test]
    fn the_version_is_the_body_hash_and_follows_a_staged_update() {
        let (_dir, mut backend, ids) = store(&[ALONE]);

        let read = backend.get_item("cal", &ids[0]).unwrap();
        let listed = backend
            .list_items("cal", CalendarItemQuery::default())
            .unwrap()
            .items;
        let etag = read.etag.clone().unwrap();
        assert_eq!(listed[0].etag.as_deref(), Some(etag.as_str()));
        assert_eq!(etag, backend.client.reader.hash(ALONE.as_bytes()).0);

        let edited = ALONE.replace("SUMMARY:", "SUMMARY:Edited ");
        backend
            .update_item(
                "cal",
                &ids[0],
                edited.clone().into_bytes(),
                Some(&etag),
                ItemOrigin::User,
            )
            .unwrap();

        let moved = backend.get_item("cal", &ids[0]).unwrap().etag.unwrap();
        assert_ne!(moved, etag);
        assert_eq!(moved, backend.client.reader.hash(edited.as_bytes()).0);
    }

    /// A write naming a version the item no longer has stages nothing,
    /// and says so with the stable prefix.
    /// The refusals a caller acts on carry their stable code.
    #[test]
    fn a_stale_if_match_and_an_unsupported_occurrence_carry_their_code() {
        use pimalaya_cli::error::code_of;

        use crate::error::ErrorCode;

        let (_dir, mut backend, ids) = store(&[ALONE]);
        let err = backend
            .delete_item("cal", &ids[0], Some("stale"))
            .unwrap_err();
        assert_eq!(code_of(&err), Some(ErrorCode::PreconditionFailed));

        let (_dir, mut backend, ids) = store(&[WEEKLY]);
        let err = backend
            .cancel("cal", &ids[0], Some("20261012T090000Z"), None, None)
            .unwrap_err();
        assert_eq!(code_of(&err), Some(ErrorCode::OccurrenceUnsupported));
    }

    /// An id naming no item, a non-numeric one included, and a calendar
    /// the store does not hold carry their code.
    #[test]
    fn a_missing_item_or_calendar_carries_its_code() {
        use pimalaya_cli::error::code_of;

        use crate::error::ErrorCode;

        let (_dir, mut backend, ids) = store(&[ALONE]);

        for id in ["999", "abc@example.org"] {
            let err = backend.get_item("cal", id).unwrap_err();
            assert_eq!(code_of(&err), Some(ErrorCode::ItemNotFound), "{err:#}");
        }

        let err = backend.get_item("nope", &ids[0]).unwrap_err();
        assert_eq!(code_of(&err), Some(ErrorCode::CalendarNotFound), "{err:#}");
    }

    #[test]
    fn a_stale_if_match_refuses_the_update_and_the_delete() {
        let (_dir, mut backend, ids) = store(&[ALONE]);
        let pending = |backend: &PimdirBackend| {
            backend
                .client
                .producer()
                .unwrap()
                .pending_actions("cal")
                .unwrap()
                .len()
        };

        let update = backend.update_item(
            "cal",
            &ids[0],
            ALONE.into(),
            Some("stale"),
            ItemOrigin::User,
        );
        let err = update.unwrap_err().to_string();
        assert!(err.starts_with(PRECONDITION_FAILED), "{err}");
        assert!(err.contains("`stale`"), "{err}");

        let delete = backend.delete_item("cal", &ids[0], Some("stale"));
        assert!(
            delete
                .unwrap_err()
                .to_string()
                .starts_with(PRECONDITION_FAILED)
        );
        assert_eq!(pending(&backend), 0);

        let etag = backend.get_item("cal", &ids[0]).unwrap().etag.unwrap();
        backend
            .delete_item("cal", &ids[0], Some(&format!("\"{etag}\"")))
            .unwrap();
        assert_eq!(pending(&backend), 1);
        assert!(backend.get_item("cal", &ids[0]).is_err());
    }

    #[test]
    fn a_declared_store_names_its_single_performer() {
        let (dir, mut backend, ids) = store(&[INVITATION]);

        let mut owner = io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_source("caldav");
        let declaration: Vec<_> = io_pimdir::capability::CALENDAR
            .iter()
            .map(|name| io_pimdir::capability::PimdirCapability {
                collection: None,
                name: name.to_string(),
                support: io_pimdir::capability::PimdirSupport::Full,
                detail: None,
            })
            .collect();
        owner.declare("caldav", &declaration).unwrap();
        drop(owner);

        let queued_row = backend
            .reply("cal", &ids[0], "TENTATIVE", None, None, None)
            .unwrap();

        assert_eq!(queued_row.source, "caldav");
        assert_eq!(queued(&backend)[0].1["source"], "caldav");
    }

    #[test]
    fn a_declared_store_refuses_a_source_lacking_the_capability() {
        let (dir, mut backend, ids) = store(&[INVITATION]);

        let mut owner = io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_source("caldav");
        let declaration: Vec<_> = io_pimdir::capability::CALENDAR
            .iter()
            .map(|name| io_pimdir::capability::PimdirCapability {
                collection: None,
                name: name.to_string(),
                support: match *name == CALENDAR_CANCEL {
                    true => io_pimdir::capability::PimdirSupport::None,
                    false => io_pimdir::capability::PimdirSupport::Full,
                },
                detail: None,
            })
            .collect();
        owner.declare("caldav", &declaration).unwrap();
        drop(owner);

        assert!(backend.cancel("cal", &ids[0], None, None, None).is_err());
        assert!(queued(&backend).is_empty());
    }

    /// A weekly invitation, organised by someone else, inviting the
    /// account.
    const WEEKLY: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//t//t//EN\r\n\
        BEGIN:VEVENT\r\nUID:weekly-invite@example.org\r\nDTSTAMP:20261001T080000Z\r\n\
        DTSTART:20261005T090000Z\r\nDTEND:20261005T100000Z\r\nRRULE:FREQ=WEEKLY\r\n\
        SUMMARY:Weekly\r\nORGANIZER;CN=Alice:mailto:alice@example.org\r\n\
        ATTENDEE;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:bob@example.org\r\n\
        END:VEVENT\r\nEND:VCALENDAR\r\n";

    /// Declares every calendar capability for `caldav`, but `lacking`.
    fn declare(dir: &tempfile::TempDir, lacking: &[&str]) {
        let mut owner = io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_source("caldav");
        let declaration: Vec<_> = io_pimdir::capability::CALENDAR
            .iter()
            .map(|name| io_pimdir::capability::PimdirCapability {
                collection: None,
                name: name.to_string(),
                support: match lacking.contains(name) {
                    true => io_pimdir::capability::PimdirSupport::None,
                    false => io_pimdir::capability::PimdirSupport::Full,
                },
                detail: None,
            })
            .collect();
        owner.declare("caldav", &declaration).unwrap();
    }

    #[test]
    fn a_reply_and_a_cancel_name_one_occurrence() {
        let (dir, mut backend, ids) = store(&[WEEKLY]);
        declare(&dir, &[]);

        let reply = backend
            .reply(
                "cal",
                &ids[0],
                "ACCEPTED",
                Some("20261012T090000Z"),
                None,
                None,
            )
            .unwrap();
        backend
            .cancel("cal", &ids[0], Some("20261019T090000Z"), None, None)
            .unwrap();

        let rows = queued(&backend);
        assert_eq!(
            rows[0].1,
            serde_json::json!({
                "v": 1,
                "seq": reply.seq,
                "source": "caldav",
                "partstat": "ACCEPTED",
                "recurrence_id": "20261012T090000Z",
            })
        );
        assert_eq!(rows[1].0, "calendar-cancel");
        assert_eq!(rows[1].1["recurrence_id"], "20261019T090000Z");
    }

    #[test]
    fn an_occurrence_the_series_lacks_or_misspells_queues_nothing() {
        let (dir, mut backend, ids) = store(&[WEEKLY]);
        declare(&dir, &[]);

        let missing = backend.reply(
            "cal",
            &ids[0],
            "ACCEPTED",
            Some("20261013T090000Z"),
            None,
            None,
        );
        assert!(missing.unwrap_err().to_string().contains("no occurrence"));

        let misspelled = backend.cancel("cal", &ids[0], Some("20261012"), None, None);
        assert!(misspelled.is_err());

        assert!(queued(&backend).is_empty());
    }

    #[test]
    fn a_performer_unable_to_act_on_one_occurrence_is_named() {
        let (dir, mut backend, ids) = store(&[WEEKLY]);
        declare(
            &dir,
            &[
                io_pimdir::capability::CALENDAR_REPLY_OCCURRENCE,
                io_pimdir::capability::CALENDAR_CANCEL_OCCURRENCE,
            ],
        );

        let err = backend
            .reply(
                "cal",
                &ids[0],
                "ACCEPTED",
                Some("20261012T090000Z"),
                None,
                None,
            )
            .unwrap_err()
            .to_string();
        assert!(err.starts_with(OCCURRENCE_UNSUPPORTED), "{err}");
        assert!(err.contains("calendar.reply.occurrence"), "{err}");
        assert!(err.contains("caldav"), "{err}");

        // NOTE: the whole series stays answerable.
        backend
            .reply("cal", &ids[0], "ACCEPTED", None, None, None)
            .unwrap();
        assert_eq!(queued(&backend).len(), 1);
    }

    #[test]
    fn an_undeclared_store_refuses_one_occurrence() {
        let (_dir, mut backend, ids) = store(&[WEEKLY]);

        let err = backend
            .cancel("cal", &ids[0], Some("20261012T090000Z"), None, None)
            .unwrap_err()
            .to_string();
        assert!(err.starts_with(OCCURRENCE_UNSUPPORTED), "{err}");
        assert!(queued(&backend).is_empty());
    }

    #[test]
    fn a_series_started_before_a_window_lists_in_it() {
        let series = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//t//t//EN\r\n\
            BEGIN:VEVENT\r\nUID:weekly@example.org\r\nDTSTAMP:20200101T000000Z\r\n\
            DTSTART:20200106T090000Z\r\nRRULE:FREQ=WEEKLY\r\nSUMMARY:Weekly\r\n\
            END:VEVENT\r\nEND:VCALENDAR\r\n";
        let (_dir, mut backend, ids) = store(&[series, ALONE]);
        let range = CalendarTimeRange {
            start: Some("20261012T000000Z".into()),
            end: Some("20261019T000000Z".into()),
        };

        let items = backend
            .list_items(
                "cal",
                CalendarItemQuery {
                    range: Some(&range),
                    ..Default::default()
                },
            )
            .unwrap()
            .items;

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, ids[0]);
        assert_eq!(
            Event::occurrences(&items[0], Some(&range))[0]
                .start
                .as_deref(),
            Some("2026-10-12T09:00:00Z")
        );
    }

    #[test]
    fn an_undownloaded_series_is_windowed_from_its_start_to_its_until() {
        let range = CalendarTimeRange {
            start: Some("20261001T000000Z".into()),
            end: Some("20261101T000000Z".into()),
        };
        let series = |until: Option<&str>| {
            stored(PimdirSummary::Event(PimdirEventSummary {
                dtstart: Some(time("20200106T090000Z")),
                recurring: Some(true),
                until: until.map(Into::into),
                ..Default::default()
            }))
        };

        assert!(in_range(&mut undownloaded(), &series(None), &range));
        assert!(in_range(
            &mut undownloaded(),
            &series(Some("20261005T000000Z")),
            &range
        ));
        assert!(!in_range(
            &mut undownloaded(),
            &series(Some("20250101T000000Z")),
            &range
        ));
    }

    /// An event asking for an online meeting (Annex B.1).
    const MEETING: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//t//t//EN\r\n\
        BEGIN:VEVENT\r\nUID:meet@example.org\r\nDTSTAMP:20261001T080000Z\r\n\
        DTSTART:20261005T090000Z\r\nSUMMARY:Sync\r\nX-PIMDIR-ONLINE-MEETING:TRUE\r\n\
        END:VEVENT\r\nEND:VCALENDAR\r\n";

    /// Declares every calendar capability for `caldav`, and
    /// `calendar.online-meeting` at `support`.
    fn declare_meeting(dir: &tempfile::TempDir, support: io_pimdir::capability::PimdirSupport) {
        let mut owner = io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_source("caldav");
        let declaration: Vec<_> = io_pimdir::capability::CALENDAR
            .iter()
            .map(|name| io_pimdir::capability::PimdirCapability {
                collection: None,
                name: name.to_string(),
                support: match *name == CALENDAR_ONLINE_MEETING {
                    true => support,
                    false => io_pimdir::capability::PimdirSupport::Full,
                },
                detail: None,
            })
            .collect();
        owner.declare("caldav", &declaration).unwrap();
    }

    fn pending(backend: &PimdirBackend) -> usize {
        backend
            .client
            .producer()
            .unwrap()
            .pending_actions("cal")
            .unwrap()
            .len()
    }

    #[test]
    fn an_online_meeting_in_an_undeclared_store_queues_nothing() {
        let (_dir, mut backend, ids) = store(&[ALONE]);

        let err = backend
            .create_item("cal", MEETING.into(), ItemOrigin::User)
            .unwrap_err();
        let code = pimalaya_cli::error::code_of(&err);
        assert_eq!(code, Some(ErrorCode::Unsupported), "{err:#}");

        let err = format!("{err:#}");
        assert!(err.starts_with("Stage the pimdir action"), "{err}");
        assert!(err.contains("calendar.online-meeting"), "{err}");
        assert!(err.contains("no source of this store declares it"), "{err}");

        let asked = ALONE.replace("SUMMARY", "X-PIMDIR-ONLINE-MEETING:TRUE\r\nSUMMARY");
        let err = backend.update_item("cal", &ids[0], asked.into(), None, ItemOrigin::User);
        assert!(err.is_err());

        assert_eq!(pending(&backend), 0);
    }

    /// An iCalendar the user did not write as it stands is queued without
    /// its `X-PIMDIR-` properties (pimdir STORAGE §18): it asks for
    /// nothing, so no gate refuses it, and its online meeting is never
    /// created; one the user wrote keeps its ask whole.
    #[test]
    fn a_foreign_body_is_queued_without_its_instructions() {
        let (_dir, mut backend, ids) = store(&[ALONE]);
        let asked = ALONE.replace("SUMMARY", "X-PIMDIR-ONLINE-MEETING:TRUE\r\nSUMMARY");

        backend
            .create_item("cal", MEETING.into(), ItemOrigin::Foreign)
            .unwrap();
        backend
            .update_item(
                "cal",
                &ids[0],
                asked.clone().into(),
                None,
                ItemOrigin::Foreign,
            )
            .unwrap();

        let objects: Vec<_> = backend
            .client
            .producer()
            .unwrap()
            .pending_actions("cal")
            .unwrap()
            .into_iter()
            .map(|pending| match pending.action {
                PimdirAction::Add { object, .. } => object.unwrap(),
                PimdirAction::Update { object, .. } => object,
                action => panic!("{action:?}"),
            })
            .collect();
        let stripped = |body: &str| {
            let body = body.replace("X-PIMDIR-ONLINE-MEETING:TRUE\r\n", "");
            backend.client.reader.hash(body.as_bytes())
        };
        assert_eq!(objects, [stripped(MEETING), stripped(&asked)]);

        let read = backend.get_item("cal", &ids[0]).unwrap().contents;
        assert!(!String::from_utf8(read).unwrap().contains("X-PIMDIR-"));
    }

    /// A foreign event with no `UID` carrying an `X-PIMDIR-` property is
    /// queued under the `hash:` key of the bytes the store holds, its
    /// instructions taken out, and that key is the id reported.
    #[test]
    fn a_uidless_foreign_event_is_keyed_by_its_stripped_body() {
        let (_dir, mut backend, _) = store(&[ALONE]);
        let uidless = MEETING.replace("UID:meet@example.org\r\n", "");
        let stripped = uidless.replace("X-PIMDIR-ONLINE-MEETING:TRUE\r\n", "");

        let id = backend
            .create_item("cal", uidless.clone().into(), ItemOrigin::Foreign)
            .unwrap();

        assert!(id.starts_with("hash:"), "{id}");
        assert_ne!(id, derive(uidless.as_bytes()).link_id.0);
        assert_eq!(id, derive(stripped.as_bytes()).link_id.0);
        assert_eq!(pending(&backend), 1);
    }

    #[test]
    fn an_online_meeting_a_source_does_not_support_names_it() {
        let (dir, mut backend, _) = store(&[ALONE]);
        declare_meeting(&dir, io_pimdir::capability::PimdirSupport::None);

        let err = backend
            .create_item("cal", MEETING.into(), ItemOrigin::User)
            .unwrap_err();
        let err = format!("{err:#}");
        assert!(err.contains("source caldav does not support it"), "{err}");
        assert_eq!(pending(&backend), 0);
    }

    #[test]
    fn an_online_meeting_a_source_declares_fully_or_in_part_is_queued() {
        use io_pimdir::capability::PimdirSupport;

        for support in [PimdirSupport::Full, PimdirSupport::Partial] {
            let (dir, mut backend, ids) = store(&[ALONE]);
            declare_meeting(&dir, support);

            backend
                .create_item("cal", MEETING.into(), ItemOrigin::User)
                .unwrap();

            let asked = ALONE.replace("SUMMARY", "X-PIMDIR-ONLINE-MEETING:TRUE\r\nSUMMARY");
            backend
                .update_item("cal", &ids[0], asked.into(), None, ItemOrigin::User)
                .unwrap();

            assert_eq!(pending(&backend), 2, "{support:?}");
        }
    }

    #[test]
    fn an_online_meeting_the_event_already_asked_for_is_no_new_ask() {
        use io_pimdir::capability::PimdirSupport;

        // NOTE: the ask lands while the source supports it, which it
        // stops declaring afterwards.
        let (dir, mut backend, ids) = store(&[ALONE]);
        declare_meeting(&dir, PimdirSupport::Full);
        let asked = ALONE.replace("SUMMARY", "X-PIMDIR-ONLINE-MEETING:TRUE\r\nSUMMARY");
        backend
            .update_item("cal", &ids[0], asked.clone().into(), None, ItemOrigin::User)
            .unwrap();

        let mut owner = io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_source("caldav");
        owner.drain().unwrap();
        drop(owner);
        declare_meeting(&dir, PimdirSupport::None);
        assert_eq!(pending(&backend), 0);

        let edited = asked.replace("SUMMARY:Focus", "SUMMARY:Deep focus");
        backend
            .update_item("cal", &ids[0], edited.into(), None, ItemOrigin::User)
            .unwrap();

        assert_eq!(pending(&backend), 1);
    }

    /// An update is pushed by the sources binding its item, so another
    /// source syncing the calendar and declaring the capability does not
    /// make the ask one somebody acts on.
    #[test]
    fn an_online_meeting_on_an_update_is_gated_by_the_item_sources() {
        use io_pimdir::{
            capability::{CALENDAR, PimdirCapability, PimdirSupport},
            change::PimdirWriteOp,
            collection::{PimdirCheckpoint, PimdirCollectionId},
            coroutine::PimdirYield,
        };

        let (dir, mut backend, ids) = store(&[ALONE]);

        // NOTE: a checkpoint makes `gcal` sync the calendar without
        // binding the item, which `caldav` alone binds and declares
        // nothing for.
        let mut owner = io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_source("gcal");
        owner
            .service(PimdirYield::WantsWrite(vec![
                PimdirWriteOp::SetCheckpoint {
                    collection: PimdirCollectionId("cal".into()),
                    checkpoint: PimdirCheckpoint(b"token".to_vec()),
                },
            ]))
            .unwrap()
            .unwrap();
        let declaration: Vec<_> = CALENDAR
            .iter()
            .map(|name| PimdirCapability {
                collection: None,
                name: name.to_string(),
                support: PimdirSupport::Full,
                detail: None,
            })
            .collect();
        owner.declare("gcal", &declaration).unwrap();
        drop(owner);

        let asked = ALONE.replace("SUMMARY", "X-PIMDIR-ONLINE-MEETING:TRUE\r\nSUMMARY");
        let err = backend
            .update_item("cal", &ids[0], asked.into(), None, ItemOrigin::User)
            .unwrap_err();
        let err = format!("{err:#}");

        assert!(err.contains("calendar.online-meeting"), "{err}");
        assert_eq!(pending(&backend), 0);
    }

    /// A calendar whose source takes no to-do refuses one, created or
    /// edited in, naming the way out.
    #[test]
    fn a_to_do_into_an_events_only_calendar_names_the_way_out() {
        use io_pimdir::capability::{
            CALENDAR, CALENDAR_COMPONENT_VTODO, PimdirCapability, PimdirSupport,
        };

        let todo = |uid: &str| {
            resource(&component(
                "VTODO",
                &format!("UID:{uid}\r\nSUMMARY:Call back\r\n"),
            ))
        };
        let (dir, mut backend, ids) = store(&[ALONE]);

        let mut owner = io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_source("caldav");
        let mut declaration: Vec<_> = CALENDAR
            .iter()
            .map(|name| PimdirCapability {
                collection: None,
                name: name.to_string(),
                support: PimdirSupport::Full,
                detail: None,
            })
            .collect();
        declaration.push(PimdirCapability {
            collection: Some("cal".into()),
            name: CALENDAR_COMPONENT_VTODO.into(),
            support: PimdirSupport::None,
            detail: Some("events only".into()),
        });
        owner.declare("caldav", &declaration).unwrap();
        drop(owner);

        let created = backend
            .create_item("cal", todo("todo@example.org").into(), ItemOrigin::User)
            .unwrap_err();
        let created = format!("{created:#}");
        let updated = backend
            .update_item(
                "cal",
                &ids[0],
                todo("alone@example.org").into(),
                None,
                ItemOrigin::User,
            )
            .unwrap_err();
        let updated = format!("{updated:#}");

        for err in [created, updated] {
            assert!(err.starts_with("Stage the pimdir action:"), "{err}");
            assert!(err.contains("takes no to-do"), "{err}");
            assert!(err.contains("caldav"), "{err}");
            assert!(err.contains("events only"), "{err}");
            assert!(err.contains("calendar.component.vtodo none"), "{err}");
            assert!(err.contains("-k/--calendar"), "{err}");
        }
        assert_eq!(pending(&backend), 0);
    }

    /// `calendar.component.vtodo` is read only where a row states it
    /// (pimdir STORAGE §15.6): `none` source-wide refuses a to-do, a row on
    /// the calendar overrides the source-wide one, and a declaration
    /// stating nothing of it, as one written before it was named, refuses
    /// nothing.
    #[test]
    fn a_to_do_is_gated_only_where_a_row_states_it() {
        use io_pimdir::capability::{
            CALENDAR, CALENDAR_COMPONENT_VTODO, PimdirCapability, PimdirSupport,
        };

        let todo = resource(&component(
            "VTODO",
            "UID:todo@example.org\r\nSUMMARY:Call back\r\n",
        ));
        let row = |collection: Option<&str>, support| PimdirCapability {
            collection: collection.map(Into::into),
            name: CALENDAR_COMPONENT_VTODO.into(),
            support,
            detail: None,
        };

        let cases = [
            (vec![row(None, PimdirSupport::None)], false),
            (vec![], true),
            (vec![row(None, PimdirSupport::Full)], true),
            (
                vec![
                    row(None, PimdirSupport::None),
                    row(Some("cal"), PimdirSupport::Full),
                ],
                true,
            ),
        ];

        for (rows, accepted) in cases {
            let (dir, mut backend, _) = store(&[ALONE]);
            let mut owner = io_pimdir::client::PimdirStore::open(dir.path())
                .unwrap()
                .for_source("caldav");
            let mut declaration: Vec<_> = CALENDAR
                .iter()
                .map(|name| PimdirCapability {
                    collection: None,
                    name: name.to_string(),
                    support: PimdirSupport::Full,
                    detail: None,
                })
                .collect();
            declaration.extend(rows.clone());
            owner.declare("caldav", &declaration).unwrap();
            drop(owner);

            let created = backend.create_item("cal", todo.clone().into(), ItemOrigin::User);
            assert_eq!(created.is_ok(), accepted, "{rows:?}: {created:?}");
            assert_eq!(pending(&backend), usize::from(accepted), "{rows:?}");
        }
    }

    /// A create whose `UID` the calendar already holds, synced or queued,
    /// is refused before anything is queued: the sync engine would park
    /// it, and the user would have read it as created.
    /// A partial support the body asks for, read by the gate with the
    /// body in hand, is reported: a gate run before the body is written
    /// reads none and passes the action as if nothing were asked.
    #[test]
    fn a_partial_support_the_body_asks_for_is_reported() {
        use io_pimdir::capability::{CALENDAR, PimdirCapability, PimdirSupport};

        let (dir, backend, _) = store(&[ALONE]);
        let mut owner = io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_source("caldav");
        let declaration: Vec<_> = CALENDAR
            .iter()
            .map(|name| PimdirCapability {
                collection: None,
                name: name.to_string(),
                support: match *name == CALENDAR_SCHEDULING {
                    true => PimdirSupport::Partial,
                    false => PimdirSupport::Full,
                },
                detail: Some("no counter-proposals".into()),
            })
            .collect();
        owner.declare("caldav", &declaration).unwrap();
        drop(owner);

        let mut producer = backend.client.producer().unwrap();
        let partial = enqueue_with_body(
            &mut producer,
            "cal",
            INVITATION.as_bytes(),
            ItemOrigin::Foreign,
            |hash| PimdirAction::Add {
                link_id: None,
                flags: PimdirFlags::default(),
                object: Some(hash),
                supersedes: None,
            },
        )
        .unwrap()
        .partial;

        let named: Vec<(&str, &str)> = partial
            .iter()
            .map(|partial| (partial.source.as_str(), partial.capability.as_str()))
            .collect();
        assert_eq!(named, [("caldav", CALENDAR_SCHEDULING)]);
        assert_eq!(pending(&backend), 1);
    }

    #[test]
    fn a_create_whose_uid_is_live_or_queued_is_refused() {
        use pimalaya_cli::error::code_of;

        let (_dir, mut backend, _) = store(&[ALONE]);
        let fresh = ALONE.replace("alone@example.org", "fresh@example.org");
        backend
            .create_item("cal", fresh.clone().into(), ItemOrigin::User)
            .unwrap();

        let live = backend
            .create_item("cal", ALONE.into(), ItemOrigin::User)
            .unwrap_err();
        let queued = backend
            .create_item("cal", fresh.into(), ItemOrigin::User)
            .unwrap_err();

        assert_eq!(code_of(&live), Some(ErrorCode::ItemDuplicate));
        assert!(
            format!("{live:#}").contains("as item 1; update"),
            "{live:#}"
        );
        assert_eq!(code_of(&queued), Some(ErrorCode::ItemDuplicate));
        assert!(format!("{queued:#}").contains("queued"), "{queued:#}");
        assert_eq!(pending(&backend), 1);
    }

    /// A create the store refuses leaves no body behind: the body is
    /// written in the transaction appending its row, after every check
    /// passed, never before.
    #[test]
    fn a_refused_create_leaves_no_body_in_the_tree() {
        let (dir, mut backend, _) = store(&[ALONE]);
        let before = blob_files(dir.path());

        let again = ALONE.replace("SUMMARY:Focus", "SUMMARY:Focus again");
        backend
            .create_item("cal", again.into(), ItemOrigin::User)
            .unwrap_err();

        assert_eq!(blob_files(dir.path()), before);
        assert_eq!(pending(&backend), 0);
    }

    /// An edit names the body it was made against: when the item holds
    /// another by the time the store's owner applies it, the owner parks
    /// it rather than drop the other write, and `pimdir status` shows it.
    #[test]
    fn an_edit_whose_body_changed_underneath_is_parked() {
        let (dir, mut backend, ids) = store(&[ALONE]);
        let mine = ALONE.replace("SUMMARY:Focus", "SUMMARY:Mine");
        let theirs = ALONE.replace("SUMMARY:Focus", "SUMMARY:Theirs");

        let etag = backend.get_item("cal", &ids[0]).unwrap().etag.unwrap();
        backend
            .update_item("cal", &ids[0], mine.into(), Some(&etag), ItemOrigin::User)
            .unwrap();

        let (landed, applied, parked) = land_and_drain(&dir, &backend, &theirs, false);
        assert_eq!((applied, parked), (0, 1));

        let held = backend.get_item("cal", &ids[0]).unwrap();
        assert_eq!(held.contents, theirs.as_bytes());

        let status = backend.status().unwrap();
        assert_eq!(status.parked.len(), 1, "{:?}", status.parked);
        let parked = &status.parked[0];
        assert_eq!(parked.kind, "update");
        assert!(parked.error.contains(&etag), "{}", parked.error);
        assert!(parked.error.contains(&landed), "{}", parked.error);
    }

    /// A parked row lists in the queue's one shape (pimalaya-cli's output
    /// contract): the event it updates as `itemId`, the summary of the
    /// body it pins, its attempts and why it was parked; a pending
    /// creation its `linkId`, its attempts and `error` `null`.
    #[test]
    fn a_parked_and_a_pending_row_list_in_one_shape() {
        let (dir, mut backend, ids) = store(&[ALONE]);
        let mine = ALONE.replace("SUMMARY:Focus", "SUMMARY:Mine");
        let theirs = ALONE.replace("SUMMARY:Focus", "SUMMARY:Theirs");

        let etag = backend.get_item("cal", &ids[0]).unwrap().etag.unwrap();
        backend
            .update_item("cal", &ids[0], mine.into(), Some(&etag), ItemOrigin::User)
            .unwrap();
        land_and_drain(&dir, &backend, &theirs, false);

        let fresh = ALONE.replace("alone@example.org", "fresh@example.org");
        backend
            .create_item("cal", fresh.into(), ItemOrigin::User)
            .unwrap();

        let parked = backend.parked_rows(None).unwrap();
        let [parked] = &parked[..] else {
            panic!("{parked:?}");
        };
        assert_eq!(parked.kind, "update");
        assert_eq!(parked.link_id, None);
        assert_eq!(parked.item_id.as_deref(), Some(ids[0].as_str()));
        assert_eq!(parked.summary.as_deref(), Some("Mine"));
        assert!(!parked.unconfirmed);
        assert!(parked.error.as_ref().is_some_and(|err| err.contains(&etag)));

        let queued = backend.pending_rows(None).unwrap().0;
        let [queued] = &queued[..] else {
            panic!("{queued:?}");
        };
        assert_eq!(queued.kind, "add");
        assert_eq!(queued.link_id.as_deref(), Some("fresh@example.org"));
        assert_eq!(queued.item_id, None);
        assert_eq!(queued.attempts, 0);
        assert_eq!(queued.error, None);
    }

    /// A delete gated on a version names the body it deletes (pimdir
    /// STORAGE §15.3): when the item holds another by the time the store's
    /// owner applies it, the owner parks it rather than delete a body
    /// nobody read, and `pimdir status` shows it.
    #[test]
    fn a_delete_whose_body_changed_underneath_is_parked() {
        let (dir, mut backend, ids) = store(&[ALONE]);
        let theirs = ALONE.replace("SUMMARY:Focus", "SUMMARY:Theirs");

        let etag = backend.get_item("cal", &ids[0]).unwrap().etag.unwrap();
        backend.delete_item("cal", &ids[0], Some(&etag)).unwrap();

        let (landed, applied, parked) = land_and_drain(&dir, &backend, &theirs, false);
        assert_eq!((applied, parked), (0, 1));

        let held = backend.get_item("cal", &ids[0]).unwrap();
        assert_eq!(held.contents, theirs.as_bytes());

        let status = backend.status().unwrap();
        assert_eq!(status.parked.len(), 1, "{:?}", status.parked);
        let parked = &status.parked[0];
        assert_eq!(parked.kind, "remove");
        assert!(parked.error.contains(&etag), "{}", parked.error);
        assert!(parked.error.contains(&landed), "{}", parked.error);
    }

    /// A delete gated on no version read no body, and deletes whatever the
    /// item holds by the time the store's owner applies it.
    #[test]
    fn a_delete_naming_no_version_removes_whatever_the_item_holds() {
        let (dir, mut backend, ids) = store(&[ALONE]);
        let theirs = ALONE.replace("SUMMARY:Focus", "SUMMARY:Theirs");

        backend.delete_item("cal", &ids[0], None).unwrap();

        let (_, applied, parked) = land_and_drain(&dir, &backend, &theirs, false);
        assert_eq!((applied, parked), (1, 0));
        assert!(backend.get_item("cal", &ids[0]).is_err());
        assert!(backend.status().unwrap().parked.is_empty());
    }

    /// calendula decides no conflict, so its writes name none (pimdir
    /// STORAGE §15.3): a delete or an edit meeting one is parked by the
    /// store's owner, the item and its diverging body kept, and `pimdir
    /// status` shows it.
    #[test]
    fn a_delete_or_an_edit_over_a_conflicted_item_is_parked() {
        let theirs = ALONE.replace("SUMMARY:Focus", "SUMMARY:Theirs");
        let mine = ALONE.replace("SUMMARY:Focus", "SUMMARY:Mine");

        for action in ["remove", "update"] {
            let (dir, mut backend, ids) = store(&[ALONE]);
            match action {
                "remove" => backend.delete_item("cal", &ids[0], None).unwrap(),
                _ => backend
                    .update_item("cal", &ids[0], mine.clone().into(), None, ItemOrigin::User)
                    .unwrap(),
            }

            let (_, applied, parked) = land_and_drain(&dir, &backend, &theirs, true);
            assert_eq!((applied, parked), (0, 1), "{action}");

            let status = backend.status().unwrap();
            assert_eq!(status.parked.len(), 1, "{:?}", status.parked);
            assert_eq!(status.parked[0].kind, action);
            assert!(
                status.parked[0].error.contains("conflict"),
                "{}",
                status.parked[0].error
            );
        }
    }

    /// Lands `theirs` on the first item of `cal` while a write waits in
    /// the queue, then drains the queue: as its body, the way a sync
    /// pulling the server's own edit does, or when `conflicted` as the
    /// diverging body of a conflict the sync left on it. Answers the hash
    /// of the body landed and how many actions the drain applied and
    /// parked.
    fn land_and_drain(
        dir: &tempfile::TempDir,
        backend: &PimdirBackend,
        theirs: &str,
        conflicted: bool,
    ) -> (String, usize, usize) {
        use io_pimdir::{
            change::PimdirWriteOp,
            collection::PimdirCollectionId,
            hub::PimdirSourceId,
            object::PimdirObject,
            placement::{PimdirBase, PimdirStatus},
        };

        let mut owner = io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_source("caldav");
        let collection = PimdirCollectionId("cal".into());
        let mut placement = owner
            .load_hub("cal")
            .unwrap()
            .project(&collection, &PimdirSourceId("caldav".into()))
            .remove(0);
        let object = PimdirObject {
            hash: backend.client.reader.hash(theirs.as_bytes()),
            size: theirs.len(),
        };
        let landed = object.hash.0.clone();
        placement.origin = None;
        if conflicted {
            placement.status = PimdirStatus::Conflict;
            placement.conflict_revision = Some("2".into());
            placement.conflict_object = Some(object.hash.clone());
            placement.base = Some(PimdirBase {
                flags: placement.flags.clone(),
                revision: Some("1".into()),
                object: placement.object.clone(),
            });
        } else {
            let derived = derive(theirs.as_bytes());
            placement.object = Some(object.hash.clone());
            placement.summary = derived.summary;
            placement.sort_key = derived.sort_key;
            placement.status = PimdirStatus::Clean;
            placement.base = Some(PimdirBase {
                flags: placement.flags.clone(),
                revision: Some("2".into()),
                object: Some(object.hash.clone()),
            });
        }
        owner
            .write(vec![
                PimdirWriteOp::StoreObject {
                    object,
                    body: Some(theirs.to_owned().into_bytes()),
                },
                PimdirWriteOp::UpsertPlacement(placement),
            ])
            .unwrap();

        let report = owner.drain().unwrap();

        (landed, report.applied, report.parked)
    }

    /// A store its sync engine has not created, a database declaring
    /// tables at `user_version` 0, another program's, and a store of
    /// another format version, an earlier or a newer engine's, are refused
    /// with io-pimdir's code and a message saying what to run or upgrade,
    /// never to delete the store.
    #[test]
    fn a_store_calendula_cannot_open_carries_its_code() {
        use std::{
            fs::OpenOptions,
            io::{Seek, SeekFrom, Write},
        };

        use pimalaya_cli::error::code_of;

        let open = |dir: &tempfile::TempDir| {
            PimdirBackend::new(PimdirConfig {
                root: dir.path().to_path_buf(),
                account: None,
            })
            .err()
            .unwrap()
        };

        let created = || {
            let dir = tempfile::tempdir().unwrap();
            io_pimdir::client::PimdirStore::open(dir.path()).unwrap();
            dir
        };
        let uncreated = open(&tempfile::tempdir().unwrap());

        // NOTE: SQLite keeps `PRAGMA user_version` big-endian at offset 60
        // of the database header.
        let stamped = |version: u32| {
            let dir = created();
            let mut db = OpenOptions::new()
                .write(true)
                .open(dir.path().join("pimdir.db"))
                .unwrap();
            db.seek(SeekFrom::Start(60)).unwrap();
            db.write_all(&version.to_be_bytes()).unwrap();
            drop(db);
            open(&dir)
        };
        let foreign = stamped(0);

        // NOTE: `store_meta.format_version` is the one place the database
        // spells its version, so a version of the same length is written
        // over it in place, the page staying well-formed.
        let versioned = |version: &str| {
            let dir = created();
            let path = dir.path().join("pimdir.db");
            let ours = io_pimdir::sql::FORMAT_VERSION.as_bytes();
            assert_eq!(version.len(), ours.len());
            let mut db = std::fs::read(&path).unwrap();
            let at: Vec<usize> = db
                .windows(ours.len())
                .enumerate()
                .filter(|(_, window)| *window == ours)
                .map(|(at, _)| at)
                .collect();
            assert_eq!(at.len(), 1, "the version is spelled once");
            db[at[0]..at[0] + ours.len()].copy_from_slice(version.as_bytes());
            std::fs::write(&path, db).unwrap();
            open(&dir)
        };
        let earlier = versioned("storage-draft-12");
        let newer = versioned("storage-draft-18");

        assert_eq!(code_of(&uncreated), Some(ErrorCode::StoreUncreated));
        assert_eq!(code_of(&foreign), Some(ErrorCode::StoreForeign));
        assert_eq!(code_of(&earlier), Some(ErrorCode::StoreVersion));
        assert_eq!(code_of(&newer), Some(ErrorCode::StoreVersion));
        assert!(
            format!("{earlier:#}").contains("move its directory aside"),
            "{earlier:#}"
        );
        assert!(
            format!("{newer:#}").contains("upgrade calendula"),
            "{newer:#}"
        );
        for err in [uncreated, foreign, earlier, newer] {
            assert!(!format!("{err:#}").contains("delete"), "{err:#}");
        }
    }

    /// A reply or a cancel reads the event's body to check it is an
    /// invitation and holds the occurrence named: a body the store lists
    /// but no longer has fails them `body-pending`, as its read, rather
    /// than letting them through unchecked.
    #[test]
    fn a_reply_or_a_cancel_over_a_missing_body_queues_nothing() {
        use pimalaya_cli::error::code_of;

        let (dir, mut backend, ids) = store(&[INVITATION]);
        for blob in blob_files(dir.path()) {
            std::fs::remove_file(blob).unwrap();
        }

        let replied = backend
            .reply("cal", &ids[0], "ACCEPTED", None, None, None)
            .unwrap_err();
        let cancelled = backend
            .cancel("cal", &ids[0], None, None, None)
            .unwrap_err();

        for err in [replied, cancelled] {
            assert_eq!(code_of(&err), Some(ErrorCode::BodyPending), "{err:#}");
        }
        assert_eq!(pending(&backend), 0);
    }

    /// An item whose blob is gone holds no version: `--if-match` naming
    /// its former hash is stale, and an update naming none is queued over
    /// whatever the item holds, as cardamum does.
    #[test]
    fn a_write_over_a_missing_body_matches_no_version() {
        use pimalaya_cli::error::code_of;

        let (dir, mut backend, ids) = store(&[ALONE]);
        let former = backend.item("cal", &ids[0]).unwrap().object.unwrap().0;
        for blob in blob_files(dir.path()) {
            std::fs::remove_file(blob).unwrap();
        }

        let stale = backend
            .update_item(
                "cal",
                &ids[0],
                ALONE.as_bytes().to_vec(),
                Some(&former),
                ItemOrigin::User,
            )
            .unwrap_err();
        assert_eq!(
            code_of(&stale),
            Some(ErrorCode::PreconditionFailed),
            "{stale:#}"
        );
        let stale = backend
            .delete_item("cal", &ids[0], Some(&former))
            .unwrap_err();
        assert_eq!(
            code_of(&stale),
            Some(ErrorCode::PreconditionFailed),
            "{stale:#}"
        );
        assert_eq!(pending(&backend), 0);

        backend
            .update_item(
                "cal",
                &ids[0],
                ALONE.as_bytes().to_vec(),
                None,
                ItemOrigin::User,
            )
            .unwrap();
        assert_eq!(pending(&backend), 1);
    }

    /// A body the store cannot read lists empty beside an `item-unreadable`
    /// entry, kept on the page holding its item: the item past a page takes
    /// its entry to the next. One its blob tree no longer holds is not
    /// hydrated and not an error (pimdir GUIDE §14): it lists as an
    /// unfetched one, no entry and no version, and its read answers
    /// `body-pending`, as Himalaya reads such a message.
    #[cfg(unix)]
    #[test]
    fn a_body_the_store_cannot_serve_is_an_entry_on_its_page() {
        use pimalaya_cli::error::code_of;

        let (dir, mut backend, ids) = store(&[INVITATION, ALONE]);
        poison(&backend, &[&ids[0]]);
        let stored = backend.item("cal", &ids[1]).unwrap();
        let blob = backend.client.blobs.path(stored.object.as_ref().unwrap());
        std::fs::remove_file(blob.unwrap()).unwrap();

        let events = CalendarItemQuery {
            kind: IcalFamily::Event.kind(),
            ..Default::default()
        };
        let listed = backend.list_items("cal", events).unwrap();
        let listed_ids: Vec<&str> = listed.items.iter().map(|item| item.id.as_str()).collect();
        assert_eq!(listed_ids, [ids[0].as_str(), ids[1].as_str()]);
        assert!(listed.items.iter().all(|item| item.contents.is_empty()));
        assert_eq!(listed.items[1].etag, None);

        let [unread] = &listed.errors[..] else {
            panic!("{:?}", listed.errors);
        };
        assert_eq!(unread.id.as_deref(), Some(ids[0].as_str()));
        assert_eq!(unread.code, Some(ErrorCode::ItemUnreadable));
        assert!(
            unread.error.starts_with("Cannot read the body"),
            "{unread:?}"
        );
        assert!(!unread.sources.is_empty(), "{unread:?}");

        let err = backend.get_item("cal", &ids[1]).unwrap_err();
        assert_eq!(code_of(&err), Some(ErrorCode::BodyPending), "{err:#}");
        assert_eq!(
            err.to_string(),
            format!(
                "Item `{}` in calendar `cal` is not downloaded yet (body not \
                 fetched); run a sync to hydrate it",
                ids[1]
            )
        );

        let second = CalendarItemQuery {
            skip: 1,
            take: Some(1),
            ..events
        };
        let listed = backend.list_items("cal", second).unwrap();
        assert!(listed.errors.is_empty(), "{:?}", listed.errors);
        assert_eq!(listed.items[0].id, ids[1]);

        let account = AccountConfig {
            pimdir: Some(PimdirConfig {
                root: dir.path().to_path_buf(),
                account: None,
            }),
            ..Default::default()
        };
        let mut client =
            CalendarClient::new(Config::default(), "a".into(), account, Backend::Auto).unwrap();
        let page = client.list_items("cal", Some(1), Some(1), events).unwrap();
        assert!(page.more);
        assert_eq!(page.errors.len(), 1, "{:?}", page.errors);
        assert_eq!(page.errors[0].id.as_deref(), Some(ids[0].as_str()));
    }

    /// A hydrated body that does not parse may be of any kind, so an event
    /// listing keeps it for `event list` to name.
    #[test]
    fn a_body_that_does_not_parse_passes_the_kind_filter() {
        let (_dir, mut backend, ids) = store(&[ALONE]);
        let mut item = backend.get_item("cal", &ids[0]).unwrap();
        let stored = backend.item("cal", &ids[0]).unwrap();
        item.contents = b"not a calendar".to_vec();

        assert!(is_kind(&item, &stored, IcalComponentKind::VEvent));
        assert!(is_kind(&item, &stored, IcalComponentKind::VTodo));
        assert!(in_range(&mut item, &stored, &CalendarTimeRange::default()));
    }

    /// Every file of the store's blob tree, a write's temporary one
    /// included.
    fn blob_files(dir: &std::path::Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let mut dirs = vec![dir.join("objects")];

        while let Some(dir) = dirs.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };

            for entry in entries {
                let path = entry.unwrap().path();
                match path.is_dir() {
                    true => dirs.push(path),
                    false => files.push(path),
                }
            }
        }

        files.sort();
        files
    }

    /// TWN-3: a write to a calendar no source syncs is refused, nothing
    /// queued, since nobody would push it.
    #[test]
    fn a_write_no_source_would_push_is_refused() {
        use pimalaya_cli::error::code_of;

        let dir = crate::pimdir::client::store_dir();
        let store = io_pimdir::client::PimdirStore::open(dir.path()).unwrap();
        store.ensure_collection("cal", CALENDAR_KIND).unwrap();
        drop(store);
        let mut backend = PimdirBackend::new(PimdirConfig {
            root: dir.path().to_path_buf(),
            account: None,
        })
        .unwrap();

        let err = backend
            .create_item("cal", ALONE.as_bytes().to_vec(), ItemOrigin::User)
            .unwrap_err();

        assert_eq!(code_of(&err), Some(ErrorCode::NoPerformer), "{err:#}");
        assert!(err.to_string().contains("run a sync first"), "{err:#}");
        assert!(queued(&backend).is_empty());
    }

    /// A write the next sync pushes is confirmed as queued, never as done.
    #[test]
    fn a_write_is_confirmed_as_queued() {
        use clap::Parser;
        use pimalaya_cli::printer::Printer;

        use crate::{
            backend::Backend,
            config::{AccountConfig, Config},
            shared::{
                client::CalendarClient,
                event::{
                    create::EventCreateCommand, delete::EventDeleteCommand,
                    update::EventUpdateCommand,
                },
            },
        };

        /// A printer keeping what it is handed, as text.
        #[derive(Default)]
        struct Capture(String);

        impl Printer for Capture {
            fn out<T: std::fmt::Display + serde::Serialize>(&mut self, data: T) -> Result<()> {
                self.0.push_str(&data.to_string());
                Ok(())
            }

            fn event<T: std::fmt::Display + serde::Serialize>(&mut self, _event: T) -> Result<()> {
                Ok(())
            }

            fn out_bytes(&mut self, _bytes: pimalaya_cli::output::Bytes) -> Result<()> {
                Ok(())
            }

            fn is_json(&self) -> bool {
                false
            }
        }

        let (dir, _, ids) = store(&[ALONE]);
        let client = || {
            let account = AccountConfig {
                pimdir: Some(PimdirConfig {
                    root: dir.path().to_path_buf(),
                    account: None,
                }),
                ..Default::default()
            };
            CalendarClient::new(Config::default(), "a".into(), account, Backend::Auto).unwrap()
        };
        let mut out = Capture::default();

        EventCreateCommand::try_parse_from(["create", "-k", "cal", "--summary", "Sync"])
            .unwrap()
            .execute(&mut out, &mut client())
            .unwrap();
        EventUpdateCommand::try_parse_from(["update", "-k", "cal", &ids[0], "--summary", "Sync"])
            .unwrap()
            .execute(&mut out, &mut client())
            .unwrap();
        EventDeleteCommand::try_parse_from(["delete", "-k", "cal", &ids[0]])
            .unwrap()
            .execute(&mut out, &mut client())
            .unwrap();

        let lines: Vec<&str> = out.0.lines().collect();
        assert_eq!(lines.len(), 3, "{lines:?}");
        assert!(
            lines[0].ends_with("` queued for the next sync"),
            "{lines:?}"
        );
        assert_eq!(
            lines[1],
            format!("Event `{}` update queued for the next sync", ids[0])
        );
        assert_eq!(
            lines[2],
            format!("Event `{}` deletion queued for the next sync", ids[0])
        );
    }

    /// A collector holds the staging lock exclusively across its file
    /// walk and deletes every body no row pins (pimdir STORAGE §8), so a
    /// body landing in the tree while it runs is one it may delete before
    /// the enqueue pins it: the queued action then names a body that is
    /// gone. A write waits for the collector before writing anything.
    #[test]
    fn a_write_puts_no_body_in_the_tree_while_a_collector_runs() {
        let (dir, mut backend, ids) = store(&[ALONE]);
        let before = blob_files(dir.path());

        let created = ALONE.replace("alone@example.org", "new@example.org");
        let edited = ALONE.replace("SUMMARY:Focus", "SUMMARY:Deep focus");

        std::thread::scope(|scope| {
            // NOTE: what `collect_garbage` takes, held here across the
            // window, and released by unwinding if the assertion fails.
            let collector = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(dir.path().join("objects.lock"))
                .unwrap();
            collector.lock().unwrap();

            let writer = scope.spawn(|| {
                backend
                    .create_item("cal", created.into(), ItemOrigin::User)
                    .unwrap();
                backend
                    .update_item("cal", &ids[0], edited.into(), None, ItemOrigin::User)
                    .unwrap();
            });

            let deadline = std::time::Instant::now() + std::time::Duration::from_millis(300);
            while std::time::Instant::now() < deadline && blob_files(dir.path()) == before {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            assert_eq!(
                blob_files(dir.path()),
                before,
                "a body landed while the collector held the staging lock"
            );

            drop(collector);
            writer.join().unwrap();
        });

        let mut owner = io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_source("caldav");
        owner.drain().unwrap();
        owner.collect_garbage().unwrap();
        drop(owner);

        let mut backend = PimdirBackend::new(PimdirConfig {
            root: dir.path().to_path_buf(),
            account: None,
        })
        .unwrap();
        let items = backend
            .list_items("cal", CalendarItemQuery::default())
            .unwrap()
            .items;
        assert_eq!(items.len(), 2);
        for item in items {
            backend.get_item("cal", &item.id).unwrap();
        }
    }

    /// A whole iCalendar resource around `components`.
    fn resource(components: &str) -> String {
        format!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//t//t//EN\r\n{components}END:VCALENDAR\r\n"
        )
    }

    /// One component of `kind` carrying `lines`.
    fn component(kind: &str, lines: &str) -> String {
        format!("BEGIN:{kind}\r\nDTSTAMP:20260101T000000Z\r\n{lines}END:{kind}\r\n")
    }

    /// A calendar mixing every shape a window, a kind or a UID has to
    /// answer: singles, all-day and long events, open and closed series,
    /// an override moved into a window from an identity past it, a
    /// resource holding overrides and no master, an escaped `UID`, to-dos, journals and a resource no
    /// summary table takes.
    fn mixed_calendar() -> Vec<String> {
        let mut bodies = Vec::new();

        for (index, day) in [3, 9, 14, 21, 28].into_iter().enumerate() {
            for month in [8, 9, 10, 11, 12] {
                bodies.push(resource(&component(
                    "VEVENT",
                    &format!(
                        "UID:single-{month}-{index}@example.org\r\nSUMMARY:Single {month} {index}\r\n\
                         DTSTART:2026{month:02}{day:02}T090000Z\r\nDTEND:2026{month:02}{day:02}T100000Z\r\n"
                    ),
                )));
            }
        }

        bodies.push(resource(&component(
            "VEVENT",
            "UID:all-day@example.org\r\nSUMMARY:All day\r\n\
             DTSTART;VALUE=DATE:20261013\r\nDTEND;VALUE=DATE:20261014\r\n",
        )));
        bodies.push(resource(&component(
            "VEVENT",
            "UID:long@example.org\r\nSUMMARY:Long\r\n\
             DTSTART:20260920T090000Z\r\nDTEND:20261020T090000Z\r\n",
        )));
        bodies.push(resource(&component(
            "VEVENT",
            "UID:weekly@example.org\r\nSUMMARY:Weekly\r\n\
             DTSTART:20200106T090000Z\r\nDTEND:20200106T100000Z\r\nRRULE:FREQ=WEEKLY\r\n",
        )));
        bodies.push(resource(&component(
            "VEVENT",
            "UID:ended@example.org\r\nSUMMARY:Ended\r\n\
             DTSTART:20250106T090000Z\r\nRRULE:FREQ=WEEKLY;UNTIL=20250301T000000Z\r\n",
        )));
        bodies.push(resource(&format!(
            "{}{}",
            component(
                "VEVENT",
                "UID:moved@example.org\r\nSUMMARY:Moved\r\n\
                 DTSTART:20250106T090000Z\r\nRRULE:FREQ=WEEKLY;COUNT=3\r\n",
            ),
            component(
                "VEVENT",
                "UID:moved@example.org\r\nSUMMARY:Moved out\r\n\
                 RECURRENCE-ID:20250120T090000Z\r\nDTSTART:20261014T090000Z\r\n",
            ),
        )));
        bodies.push(resource(&format!(
            "{}{}",
            component(
                "VEVENT",
                "UID:orphans@example.org\r\nSUMMARY:First instance\r\n\
                 RECURRENCE-ID:20250106T090000Z\r\nDTSTART:20250106T090000Z\r\n",
            ),
            component(
                "VEVENT",
                "UID:orphans@example.org\r\nSUMMARY:Later instance\r\n\
                 RECURRENCE-ID:20261012T090000Z\r\nDTSTART:20261012T090000Z\r\n",
            ),
        )));
        bodies.push(resource(&component(
            "VEVENT",
            "UID:esc\\,aped@example.org\r\nSUMMARY:Escaped\r\nDTSTART:20261016T090000Z\r\n",
        )));
        for day in [2, 12, 22] {
            bodies.push(resource(&component(
                "VTODO",
                &format!("UID:todo-{day}@example.org\r\nSUMMARY:Todo {day}\r\nDUE;VALUE=DATE:202610{day:02}\r\n"),
            )));
            bodies.push(resource(&component(
                "VJOURNAL",
                &format!("UID:journal-{day}@example.org\r\nSUMMARY:Journal {day}\r\nDTSTART:202610{day:02}T080000Z\r\n"),
            )));
        }
        bodies.push(resource(&component(
            "VTODO",
            "UID:undated-todo@example.org\r\nSUMMARY:Someday\r\n",
        )));
        bodies.push(resource(&component(
            "VFREEBUSY",
            "UID:busy@example.org\r\nDTSTART:20261012T000000Z\r\nDTEND:20261013T000000Z\r\n",
        )));

        bodies
    }

    /// The listing as it was before it streamed: every summary, then
    /// every body, then the filters, then the page cut in memory.
    fn eager(
        backend: &PimdirBackend,
        calendar_id: &str,
        query: CalendarItemQuery<'_>,
    ) -> Result<Vec<CalendarItem>> {
        let mut all = Vec::new();
        backend.scan(calendar_id, |stored| {
            all.push(stored);
            Ok(ControlFlow::Continue(()))
        })?;

        let mut items = Vec::new();
        for stored in &all {
            let (mut item, error) = backend.item_from(calendar_id, stored);
            if let Some(error) = error {
                bail!("{}", error.error);
            }
            if query.kind.is_some_and(|kind| !is_kind(&item, stored, kind)) {
                continue;
            }
            if query
                .range
                .is_some_and(|range| !in_range(&mut item, stored, range))
            {
                continue;
            }
            items.push(item);
        }

        Ok(crate::shared::client::window(items, query.skip, query.take))
    }

    /// What `event search` prints for `uid` out of `items`.
    fn found(items: &[CalendarItem], uid: &str) -> Vec<Event> {
        items
            .iter()
            .flat_map(Event::project)
            .filter(|event| event.uid.as_deref() == Some(uid))
            .collect()
    }

    /// The windows a listing is asked for: none, a month, a week, and
    /// open on either side.
    fn ranges() -> Vec<Option<CalendarTimeRange>> {
        let range = |start: Option<&str>, end: Option<&str>| {
            Some(CalendarTimeRange {
                start: start.map(Into::into),
                end: end.map(Into::into),
            })
        };

        vec![
            None,
            range(Some("20260801T000000Z"), Some("20260901T000000Z")),
            range(Some("20261012T000000Z"), Some("20261019T000000Z")),
            range(None, Some("20260910T000000Z")),
            range(Some("20261201T000000Z"), None),
        ]
    }

    /// Every listing a command asks for comes out as it did when the
    /// whole collection was read first: same items, same order, same
    /// bodies and versions, page by page, across batch boundaries.
    fn assert_streams_as_it_listed(backend: &mut PimdirBackend) {
        let kinds = [
            None,
            Some(IcalComponentKind::VEvent),
            Some(IcalComponentKind::VTodo),
            Some(IcalComponentKind::VJournal),
        ];
        let windows = [
            (0, None),
            (0, Some(3)),
            (3, Some(4)),
            (15, Some(6)),
            (42, Some(8)),
            (495, Some(6)),
            (0, Some(0)),
            (1, None),
        ];
        let mut listed = 0;

        for kind in kinds {
            for range in ranges() {
                for (skip, take) in windows {
                    let query = CalendarItemQuery {
                        skip,
                        take,
                        range: range.as_ref(),
                        kind,
                        uid: None,
                    };
                    let streamed = backend.list_items("cal", query).unwrap().items;
                    let expected = eager(backend, "cal", query).unwrap();
                    assert_eq!(streamed, expected, "{query:?}");
                    listed += streamed.len();
                }
            }
        }

        // NOTE: the matrix lists something, or it proves nothing.
        assert!(listed > 300, "{listed}");

        for uid in [
            "moved@example.org",
            "orphans@example.org",
            "weekly@example.org",
            "esc,aped@example.org",
            "esc\\,aped@example.org",
            "todo-12@example.org",
            "renamed@example.org",
            "missing@example.org",
        ] {
            let query = CalendarItemQuery {
                kind: IcalFamily::Event.kind(),
                uid: Some(uid),
                ..Default::default()
            };
            let streamed = backend.list_items("cal", query).unwrap().items;
            let expected = eager(backend, "cal", query).unwrap();
            assert_eq!(found(&streamed, uid), found(&expected, uid), "{uid}");

            // NOTE: a window ignores the hint rather than shift.
            let paged = CalendarItemQuery {
                skip: 4,
                take: Some(4),
                ..query
            };
            assert_eq!(
                backend.list_items("cal", paged).unwrap().items,
                eager(backend, "cal", paged).unwrap(),
                "{uid}"
            );
        }
    }

    #[test]
    fn a_streamed_listing_lists_what_the_whole_read_listed() {
        let bodies = mixed_calendar();
        let bodies: Vec<&str> = bodies.iter().map(String::as_str).collect();
        let (_dir, mut backend, ids) = store(&bodies);
        assert_eq!(ids.len(), bodies.len());
        assert!(ids.len() > 4 * SCAN_BATCH);

        assert_streams_as_it_listed(&mut backend);

        // NOTE: the overrides are there to be found.
        let moved = backend
            .list_items(
                "cal",
                CalendarItemQuery {
                    kind: IcalFamily::Event.kind(),
                    uid: Some("moved@example.org"),
                    ..Default::default()
                },
            )
            .unwrap()
            .items;
        assert_eq!(found(&moved, "moved@example.org").len(), 2);
        let week = ranges().remove(2).unwrap();
        let window = CalendarItemQuery {
            range: Some(&week),
            kind: IcalFamily::Event.kind(),
            ..Default::default()
        };
        let summaries: Vec<String> = backend
            .list_items("cal", window)
            .unwrap()
            .items
            .iter()
            .flat_map(|item| Event::occurrences(item, Some(&week)))
            .filter_map(|event| event.summary)
            .collect();
        for expected in ["Moved out", "Later instance", "Weekly", "Long", "All day"] {
            assert!(
                summaries.iter().any(|summary| summary == expected),
                "{expected}: {summaries:?}"
            );
        }
    }

    /// A pending update repoints the body while the summary still
    /// describes the old one, so the summary rules nothing out for it:
    /// an event turned to-do lists as a to-do, and a new `UID` is found.
    #[test]
    fn a_staged_update_is_read_from_its_new_body() {
        let bodies = mixed_calendar();
        let bodies: Vec<&str> = bodies.iter().map(String::as_str).collect();
        let (_dir, mut backend, _) = store(&bodies);
        let mut id_of = |uid: &str| {
            let items = backend
                .list_items("cal", CalendarItemQuery::default())
                .unwrap()
                .items;
            let item = items
                .into_iter()
                .find(|item| found(std::slice::from_ref(item), uid).len() == 1);
            item.unwrap().id
        };
        let ids = [
            id_of("single-8-0@example.org"),
            id_of("single-9-0@example.org"),
        ];

        let retyped = resource(&component(
            "VTODO",
            "UID:single-8-0@example.org\r\nSUMMARY:Now a todo\r\nDUE;VALUE=DATE:20261013\r\n",
        ));
        backend
            .update_item("cal", &ids[0], retyped.into_bytes(), None, ItemOrigin::User)
            .unwrap();
        let renamed = resource(&component(
            "VEVENT",
            "UID:renamed@example.org\r\nSUMMARY:Renamed\r\nDTSTART:20261013T090000Z\r\n",
        ));
        backend
            .update_item("cal", &ids[1], renamed.into_bytes(), None, ItemOrigin::User)
            .unwrap();

        assert_streams_as_it_listed(&mut backend);

        let todos = backend
            .list_items(
                "cal",
                CalendarItemQuery {
                    kind: Some(IcalComponentKind::VTodo),
                    ..Default::default()
                },
            )
            .unwrap()
            .items;
        assert!(todos.iter().any(|item| item.id == ids[0]));
        let renamed = backend
            .list_items(
                "cal",
                CalendarItemQuery {
                    kind: IcalFamily::Event.kind(),
                    uid: Some("renamed@example.org"),
                    ..Default::default()
                },
            )
            .unwrap()
            .items;
        assert_eq!(found(&renamed, "renamed@example.org")[0].id, ids[1]);
    }

    /// Makes the bodies of `ids` unreadable: each blob loses every
    /// permission, so any read of it is an entry of the listing's errors,
    /// and fails the eager one.
    ///
    /// A blob turned into a directory would do too: the store reads such
    /// an entry as no body at all (pimdir STORAGE §5), the state of a
    /// missing one. Run as root, the open would still succeed.
    fn poison(backend: &PimdirBackend, ids: &[&String]) {
        use std::os::unix::fs::PermissionsExt;

        for id in ids {
            let stored = backend.item("cal", id).unwrap();
            let path = backend
                .client
                .blobs
                .path(stored.object.as_ref().unwrap())
                .unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();
        }
    }

    /// The items of a listing that read no poisoned body: one it read
    /// would be an entry of its errors.
    fn unpoisoned(list: CalendarItemList) -> Vec<CalendarItem> {
        assert!(list.errors.is_empty(), "{:?}", list.errors);
        list.items
    }

    /// A listing reads the bodies it returns and those its filters need,
    /// and no other: not those past a full page, not those before a page
    /// no filter narrows, not those the summary rules out by kind or by
    /// `UID`.
    #[test]
    fn a_listing_reads_no_body_it_does_not_need() {
        let bodies = mixed_calendar();
        let bodies: Vec<&str> = bodies.iter().map(String::as_str).collect();
        let (_dir, mut backend, _) = store(&bodies);
        let all = || CalendarItemQuery::default();
        let order: Vec<String> = unpoisoned(backend.list_items("cal", all()).unwrap())
            .into_iter()
            .map(|item| item.id)
            .collect();
        let kind_of = |id: &String| {
            let item = backend.get_item("cal", id).unwrap();
            [
                IcalComponentKind::VEvent,
                IcalComponentKind::VTodo,
                IcalComponentKind::VJournal,
            ]
            .into_iter()
            .find(|kind| may_hold_kind(&item.contents, *kind))
        };
        let kinds: Vec<_> = order.iter().map(kind_of).collect();
        let uid_of = |id: &String| {
            let item = backend.get_item("cal", id).unwrap();
            Event::project(&item)
                .first()
                .and_then(|event| event.uid.clone())
        };
        let uids: Vec<_> = order.iter().map(uid_of).collect();

        // Past a full page, and before a page nothing filters.
        let page = |page: usize, page_size: usize| CalendarItemQuery {
            skip: (page - 1) * page_size,
            take: Some(page_size),
            ..all()
        };
        let first: Vec<String> = unpoisoned(backend.list_items("cal", page(1, 3)).unwrap())
            .into_iter()
            .map(|item| item.id)
            .collect();
        let events = CalendarItemQuery {
            kind: IcalFamily::Event.kind(),
            ..page(1, 3)
        };
        let first_events = unpoisoned(backend.list_items("cal", events).unwrap());
        poison(
            &backend,
            &order
                .iter()
                .skip(3)
                .filter(|id| !first_events.iter().any(|item| &item.id == *id))
                .take(10)
                .collect::<Vec<_>>(),
        );
        assert_eq!(
            unpoisoned(backend.list_items("cal", page(1, 3)).unwrap())
                .into_iter()
                .map(|item| item.id)
                .collect::<Vec<_>>(),
            first
        );
        assert_eq!(
            unpoisoned(backend.list_items("cal", events).unwrap()),
            first_events
        );
        assert!(eager(&backend, "cal", page(1, 3)).is_err());

        // NOTE: each case poisons a store of its own, built alike.
        let fresh = || {
            let (dir, backend, ids) = store(&bodies);
            assert_eq!(ids, order);
            (dir, backend)
        };

        let (_dir, mut backend) = fresh();
        poison(&backend, &order.iter().take(4).collect::<Vec<_>>());
        assert_eq!(
            unpoisoned(backend.list_items("cal", page(2, 4)).unwrap())
                .into_iter()
                .map(|item| item.id)
                .collect::<Vec<_>>(),
            order[4..8]
        );
        assert!(eager(&backend, "cal", page(2, 4)).is_err());

        // Every event, under a to-do or a journal listing.
        let (_dir, mut backend) = fresh();
        let events: Vec<&String> = order
            .iter()
            .zip(&kinds)
            .filter(|(_, kind)| **kind == Some(IcalComponentKind::VEvent))
            .map(|(id, _)| id)
            .collect();
        poison(&backend, &events);
        for (kind, count) in [
            (IcalComponentKind::VTodo, 4),
            (IcalComponentKind::VJournal, 3),
        ] {
            let query = CalendarItemQuery {
                kind: Some(kind),
                ..all()
            };
            assert_eq!(
                unpoisoned(backend.list_items("cal", query).unwrap()).len(),
                count
            );
            assert!(eager(&backend, "cal", query).is_err());
        }

        // Every event carrying another UID, under a find. The escaped one
        // is read whatever its spelling, as the hint leaves it alone.
        let (_dir, mut backend) = fresh();
        let others: Vec<&String> = order
            .iter()
            .zip(&uids)
            .filter(|(_, uid)| uid.as_deref().is_some_and(|uid| uid != "moved@example.org"))
            .filter(|(_, uid)| !uid.as_deref().is_some_and(|uid| uid.contains("aped")))
            .map(|(id, _)| id)
            .collect();
        assert_eq!(others.len(), 30);
        poison(&backend, &others);
        let find = CalendarItemQuery {
            kind: IcalFamily::Event.kind(),
            uid: Some("moved@example.org"),
            ..all()
        };
        let moved = unpoisoned(backend.list_items("cal", find).unwrap());
        assert_eq!(found(&moved, "moved@example.org").len(), 2);
        assert!(eager(&backend, "cal", find).is_err());
    }
}
