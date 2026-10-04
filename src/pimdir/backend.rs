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
//! Writes append one action to the store's queue (pimdir SPEC 15.1):
//! the body reaches the blob tree first, then the row pinning it. The
//! owner, a sync, derives the summary from that body, applies and
//! pushes it, while the reader folds the pending queue over its reads
//! so a staged change shows before that.
//!
//! Calendars come from the sync, so the collection verbs (create,
//! update, delete) are not served here: a cache does not invent
//! collections its source does not have.
//!
//! Ids are the store's public `seq`, stable across every collection an
//! item is filed in, never the internal link id.

use std::{io::Write, path::PathBuf};

use anyhow::{Result, anyhow, bail};
use ical::component::IcalComponentKind;
use io_pimdir::{
    capability::{
        CALENDAR_CANCEL, CALENDAR_CANCEL_OCCURRENCE, CALENDAR_REPLY, CALENDAR_REPLY_OCCURRENCE,
        CALENDAR_SCHEDULING,
    },
    client::{
        PimdirError,
        reader::{PimdirCollection, PimdirItem},
    },
    codec::PimdirAction,
    intent::{PimdirIntentItem, PimdirInvitation, PimdirPartstat},
    object::PimdirObject,
    placement::PimdirFlags,
    summary::{
        PimdirSummary,
        calendar::{PimdirTime, derive},
    },
};
use log::warn;
use pimalaya_cli::printer::Printer;
use pimalaya_config::toml::TomlConfig;

use crate::{
    account::context::Account,
    cli::load_config,
    config::PimdirConfig,
    error::{CodedError, ErrorCode},
    pimdir::{
        client::PimdirClient,
        status::{PimdirCalendarStatus, PimdirStatusOutput},
    },
    shared::{
        calendar::{Calendar, CalendarDiff},
        client::paginate,
        event::Event,
        ical::holds_kind,
        item::{CalendarItem, CalendarItemQuery, CalendarTimeRange},
    },
};

/// The media type a pimdir collection carries to be a calendar.
const CALENDAR_KIND: &str = "text/calendar";

/// How many items to pull per keyset page when scanning a collection.
const SCAN_BATCH: usize = 500;

/// The shared-API glue over a pimdir store.
pub struct PimdirBackend {
    client: PimdirClient,
    /// The `calendar.default` of the account, when built from one.
    calendar_default: Option<String>,
}

impl PimdirBackend {
    /// Opens the store at the configured root.
    pub fn new(config: PimdirConfig) -> Result<Self> {
        Ok(Self {
            client: PimdirClient::new(config)?,
            calendar_default: None,
        })
    }

    /// Loads the configuration, picks the active account, then opens
    /// the store.
    ///
    /// Bails when the account carries no `[pimdir]` block.
    pub fn build(
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
    ) -> Result<Self> {
        let mut config = load_config(printer, config_paths)?;
        let (name, mut account_config) = config
            .take_account(account_name)?
            .ok_or_else(|| anyhow!("Cannot find account"))?;

        let pimdir_config = account_config
            .pimdir
            .take()
            .ok_or_else(|| anyhow!("pimdir configuration is missing for account `{name}`"))?;

        let account = Account::from(config).merge(Account::from(account_config));

        let mut backend = Self::new(pimdir_config)?;
        backend.calendar_default = account.calendar_default;
        Ok(backend)
    }

    /// The calendar a `pimdir` command operates on: the `-k/--calendar`
    /// flag, then `calendar.default`.
    pub fn calendar_id(&self, flag: Option<String>) -> Result<String> {
        flag.or_else(|| self.calendar_default.clone())
            .ok_or_else(|| {
                anyhow!("Missing calendar id; pass -k/--calendar or set calendar.default")
            })
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
            })
            .collect();

        calendars.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(calendars)
    }

    /// Refuses to create a calendar.
    ///
    /// Declaring a collection is an owner write (pimdir SPEC 8) and
    /// this backend is a producer, and a collection no sync knows about
    /// is one no sync would carry.
    pub fn create_calendar(
        &mut self,
        _id: &str,
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
    pub fn list_items(
        &mut self,
        calendar_id: &str,
        query: CalendarItemQuery<'_>,
    ) -> Result<Vec<CalendarItem>> {
        self.known_collection(calendar_id)?;

        let mut items = Vec::new();

        for stored in self.scan_items(calendar_id)? {
            let item = self.item_from(calendar_id, &stored)?;

            if let Some(kind) = query.kind
                && !is_kind(&item, &stored, kind)
            {
                continue;
            }

            if let Some(range) = query.range
                && !in_range(&item, &stored, range)
            {
                continue;
            }

            items.push(item);
        }

        Ok(paginate(items, query.page, query.page_size))
    }

    /// Reads one item's bytes from its content-addressed blob.
    ///
    /// Fails with a clear "body not fetched" when the item is not
    /// hydrated: a state to resolve with a sync, not a missing item.
    pub fn get_item(&mut self, calendar_id: &str, item_id: &str) -> Result<CalendarItem> {
        self.known_collection(calendar_id)?;

        let stored = self.item(calendar_id, item_id)?;

        let Some(hash) = stored.object else {
            return Err(CodedError::new(
                ErrorCode::BodyPending,
                format!(
                    "Item `{item_id}` in calendar `{calendar_id}` is not downloaded yet \
                     (body not fetched); run a sync to hydrate it"
                ),
            )
            .into());
        };

        let contents = self.client.blobs.get(&hash)?.ok_or_else(|| {
            anyhow!("Body blob missing for item `{item_id}` in calendar `{calendar_id}`")
        })?;

        Ok(CalendarItem {
            id: stored.seq.to_string(),
            calendar_id: calendar_id.to_owned(),
            etag: Some(hash.0),
            contents,
        })
    }

    /// Stages a locally-authored item as an `add` the next sync pushes.
    ///
    /// Returns the item's link id, its `UID` under the store's own
    /// derivation (pimdir STORAGE Annex A.3): a queued create carries no
    /// public `seq` until the store's owner applies it, so there is no
    /// store-assigned id to report yet.
    pub fn create_item(&mut self, calendar_id: &str, contents: Vec<u8>) -> Result<String> {
        self.known_collection(calendar_id)?;

        let link_id = derive(&contents).link_id;
        let object = self.stage_body(&contents)?;

        let action = PimdirAction::Add {
            link_id: Some(link_id.clone()),
            flags: PimdirFlags::default(),
            object: Some(object.hash.clone()),
        };
        self.enqueue(calendar_id, &action, Some(&object))?;

        Ok(link_id.0)
    }

    /// Stages a body replacement as an `update` the next sync pushes.
    ///
    /// `if_match` gates the staging on the item's version, the store's
    /// hash of its body with the pending queue folded in. The engine
    /// still three-way merges the update against the base body it
    /// recorded at sync time: the gate is the caller's, the merge the
    /// store's.
    pub fn update_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<()> {
        self.known_collection(calendar_id)?;

        let stored = self.item(calendar_id, item_id)?;
        check_version(&stored, calendar_id, item_id, if_match)?;
        let seq = stored.seq;
        let object = self.stage_body(&contents)?;

        let action = PimdirAction::Update {
            seq,
            object: object.hash.clone(),
        };
        self.enqueue(calendar_id, &action, Some(&object))
    }

    /// Stages a `remove` action: a tombstone, then a server-side delete.
    ///
    /// `if_match` gates it as it gates [`update_item`](Self::update_item).
    pub fn delete_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        if_match: Option<&str>,
    ) -> Result<()> {
        self.known_collection(calendar_id)?;

        let stored = self.item(calendar_id, item_id)?;
        check_version(&stored, calendar_id, item_id, if_match)?;
        let seq = stored.seq;
        self.enqueue(calendar_id, &PimdirAction::Remove { seq }, None)
    }

    /// Collects the accounts and per-calendar hydration state that
    /// `pimdir status` reports.
    pub fn status(&mut self) -> Result<PimdirStatusOutput> {
        let accounts = self.client.reader.list_accounts()?;
        let mut calendars = Vec::new();

        for collection in self.calendar_collections()? {
            let items = self.scan_items(&collection.id)?;
            let hydrated = items.iter().filter(|item| item.object.is_some()).count();
            let queued = self.client.reader.count_pending_creates(&collection.id)?;

            calendars.push(PimdirCalendarStatus {
                name: if collection.name.is_empty() {
                    collection.id.clone()
                } else {
                    collection.name
                },
                id: collection.id,
                total: items.len(),
                hydrated,
                queued,
            });
        }

        calendars.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(PimdirStatusOutput {
            account: self.client.account.clone(),
            accounts,
            calendars,
        })
    }

    /// The store's calendar collections.
    ///
    /// Narrowed to the configured account when the store groups several
    /// (pimdir SPEC 9.2).
    fn calendar_collections(&self) -> Result<Vec<PimdirCollection>> {
        let collections = match self.client.account.as_deref() {
            Some(account) => self
                .client
                .reader
                .list_collections_by_account(Some(account))?,
            None => self.client.reader.list_collections()?,
        };

        Ok(collections
            .into_iter()
            .filter(|collection| collection.kind.is_empty() || collection.kind == CALENDAR_KIND)
            .collect())
    }

    /// Fails unless `calendar_id` names a calendar the store knows.
    ///
    /// The read seam answers an unknown collection with an empty page
    /// and the queue takes any name, so a typo in `-k` would read as an
    /// empty calendar and stage into one nothing ever applies.
    fn known_collection(&self, calendar_id: &str) -> Result<()> {
        let mut ids: Vec<String> = self
            .calendar_collections()?
            .into_iter()
            .map(|collection| collection.id)
            .collect();

        if ids.iter().any(|id| id == calendar_id) {
            return Ok(());
        }

        ids.sort();

        bail!(
            "Calendar `{calendar_id}` not found; this account holds: {}",
            ids.join(", "),
        )
    }

    /// Pulls every live item of a collection by keyset paging, each
    /// with its summary joined.
    ///
    /// In the order the store maintains for calendars, start ascending.
    fn scan_items(&self, calendar_id: &str) -> Result<Vec<PimdirItem>> {
        let mut all = Vec::new();
        let mut cursor: Option<(String, i64)> = None;

        loop {
            let page = self.client.reader.list_summaries(
                calendar_id,
                cursor.as_ref().map(|(key, seq)| (key.as_str(), *seq)),
                SCAN_BATCH,
            )?;
            let count = page.len();

            if let Some(last) = page.last() {
                cursor = Some((last.sort_key.clone(), last.seq));
            }

            all.extend(page);

            if count < SCAN_BATCH {
                break;
            }
        }

        Ok(all)
    }

    /// The stored item behind a public item id, or a clear miss.
    ///
    /// Anything non-numeric is the internal link id or a mistyped
    /// value, and saying so beats a lookup that silently finds nothing.
    fn item(&self, calendar_id: &str, item_id: &str) -> Result<PimdirItem> {
        let seq = item_id.parse::<i64>().map_err(|_| {
            anyhow!("Invalid pimdir item id `{item_id}`: expected the number a listing showed")
        })?;

        self.client
            .reader
            .get_item(calendar_id, seq)?
            .ok_or_else(|| anyhow!("Item `{item_id}` not found in calendar `{calendar_id}`"))
    }

    /// Projects a stored item onto the shared type, body when local.
    ///
    /// Empty contents mean two things and only one is ordinary: the
    /// sync has not hydrated the item, or its object's blob is gone,
    /// an inconsistent store, logged since the row renders alike.
    fn item_from(&self, calendar_id: &str, stored: &PimdirItem) -> Result<CalendarItem> {
        let contents = match &stored.object {
            Some(hash) => match self.client.blobs.get(hash)? {
                Some(contents) => contents,
                None => {
                    warn!(
                        "body blob missing for item `{}` in calendar `{calendar_id}`, \
                         listing it without contents",
                        stored.seq
                    );
                    Vec::new()
                }
            },
            None => Vec::new(),
        };

        Ok(CalendarItem {
            id: stored.seq.to_string(),
            calendar_id: calendar_id.to_owned(),
            etag: stored.object.as_ref().map(|hash| hash.0.clone()),
            contents,
        })
    }

    /// Writes a body into the blob tree, returning the object it indexes.
    ///
    /// Durable before anything references it (pimdir SPEC 14), so the
    /// queue row appended next pins a body that is already there.
    fn stage_body(&self, contents: &[u8]) -> Result<PimdirObject> {
        // NOTE: the hash is the store's, from `store_meta.hash_algo`,
        // never one this crate picks: a body named under another
        // algorithm is one no read ever finds.
        let hash = self.client.reader.hash(contents);
        let mut writer = self.client.blobs.writer()?;
        writer.write_all(contents)?;
        let size = writer.commit(&hash)?;

        Ok(PimdirObject {
            hash,
            size: size as usize,
        })
    }

    /// Appends one action to a collection's queue.
    ///
    /// The producer is opened for this write and dropped with it.
    fn enqueue(
        &mut self,
        calendar_id: &str,
        action: &PimdirAction,
        object: Option<&PimdirObject>,
    ) -> Result<()> {
        let stage = |err: PimdirError| match err {
            PimdirError::Unsupported(refusal) if refusal.capability == CALENDAR_SCHEDULING => {
                anyhow!(
                    "Stage the pimdir action: {refusal}; to write it without notifying \
                         anyone, first mark its ORGANIZER and ATTENDEE with \
                         SCHEDULE-AGENT=NONE"
                )
            }
            err => anyhow!("Stage the pimdir action: {err}"),
        };

        let mut producer = self.client.producer()?;
        let partials = producer.check(calendar_id, action).map_err(stage)?;
        producer
            .enqueue(calendar_id, action, object)
            .map_err(stage)?;

        for partial in &partials {
            warn!("{partial}");
        }
        Ok(())
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
        if let Some(contents) = self.body(&stored)? {
            if !names(&contents, "ORGANIZER") {
                bail!(
                    "Event `{item_id}` names no ORGANIZER, so it is not an invitation \
                     and there is nobody to reply to"
                );
            }
            check_occurrence(calendar_id, &stored, &contents, recurrence_id)?;
        }

        self.intent(
            calendar_id,
            PimdirInvitation {
                source: None,
                item: PimdirIntentItem::Seq(stored.seq),
                partstat: Some(partstat),
                comment: comment.map(ToOwned::to_owned),
                recurrence_id: recurrence_id.map(ToOwned::to_owned),
            },
            source,
        )
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
        if let Some(contents) = self.body(&stored)? {
            if !names(&contents, "ATTENDEE") {
                bail!(
                    "Event `{item_id}` names no ATTENDEE, so there is nobody to notify; \
                     remove it with `event delete` instead"
                );
            }
            check_occurrence(calendar_id, &stored, &contents, recurrence_id)?;
        }

        self.intent(
            calendar_id,
            PimdirInvitation {
                source: None,
                item: PimdirIntentItem::Seq(stored.seq),
                partstat: None,
                comment: comment.map(ToOwned::to_owned),
                recurrence_id: recurrence_id.map(ToOwned::to_owned),
            },
            source,
        )
    }

    /// Appends an invitation intent, anchored on its collection (pimdir
    /// STORAGE §15.6), naming its performer.
    ///
    /// A store whose sources declare nothing predates capabilities, and
    /// its owner picks the performer as it always did, so the payload
    /// names none there unless the user did.
    fn intent(
        &mut self,
        calendar_id: &str,
        mut invitation: PimdirInvitation,
        source: Option<&str>,
    ) -> Result<PimdirQueued> {
        let capability = match invitation.partstat {
            Some(_) => CALENDAR_REPLY,
            None => CALENDAR_CANCEL,
        };
        let PimdirIntentItem::Seq(seq) = invitation.item else {
            bail!("An invitation intent of calendula names a stored item");
        };

        let mut producer = self.client.producer()?;

        let declared = producer
            .capabilities(calendar_id)
            .map_err(|err| anyhow!("Read the capabilities of `{calendar_id}`: {err}"))?
            .iter()
            .any(|source| source.declared.is_some());

        invitation.source = match (declared, source) {
            (false, source) => source.map(ToOwned::to_owned),
            (true, chosen) => match producer.performer(calendar_id, capability, chosen) {
                Ok(source) => Some(source),
                Err(PimdirError::Ambiguous { candidates, .. }) => bail!(
                    "Several sources can perform {capability} for this account ({}): \
                     choose one with --source",
                    candidates.join(", "),
                ),
                Err(err) => bail!("Find who performs {capability}: {err}"),
            },
        };

        let action = invitation
            .to_action()
            .map_err(|err| anyhow!("Queue {capability}: {err}"))?;

        let refuse = |err: PimdirError| match err {
            PimdirError::Unsupported(refusal)
                if refusal.capability == CALENDAR_REPLY_OCCURRENCE
                    || refusal.capability == CALENDAR_CANCEL_OCCURRENCE =>
            {
                let who = match refusal.source.is_empty() {
                    true => "no source of this store declares it".to_owned(),
                    false => format!("source {} does not support it", refusal.source),
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
            err => anyhow!("Queue {capability}: {err}"),
        };

        let partials = producer.check(calendar_id, &action).map_err(refuse)?;
        let row = producer
            .enqueue(calendar_id, &action, None)
            .map_err(refuse)?;

        for partial in &partials {
            warn!("{partial}");
        }

        Ok(PimdirQueued {
            row,
            seq,
            source: invitation.source,
        })
    }

    /// An item's local body, `None` when the sync has not fetched it.
    fn body(&self, stored: &PimdirItem) -> Result<Option<Vec<u8>>> {
        match &stored.object {
            Some(hash) => Ok(self.client.blobs.get(hash)?),
            None => Ok(None),
        }
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
    stored: &PimdirItem,
    calendar_id: &str,
    item_id: &str,
    if_match: Option<&str>,
) -> Result<()> {
    let Some(expected) = if_match else {
        return Ok(());
    };
    let expected = expected.trim().trim_matches('"');
    let current = stored.object.as_ref().map(|hash| hash.0.as_str());

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

/// Whether a stored item is of `kind`.
///
/// A body the sync has not downloaded still answers: the summary table
/// the store filed it in names the component a reader renders the
/// resource as (pimdir STORAGE Annex A.3 to A.5), so an
/// availability-aware listing stays one.
fn is_kind(item: &CalendarItem, stored: &PimdirItem, kind: IcalComponentKind) -> bool {
    if !item.contents.is_empty() {
        return holds_kind(&item.contents, kind);
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
fn in_range(item: &CalendarItem, stored: &PimdirItem, range: &CalendarTimeRange) -> bool {
    if !item.contents.is_empty() {
        return Event::project(item)
            .iter()
            .any(|event| range.contains(&event.start))
            || !Event::occurrences(item, Some(range)).is_empty();
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
        PimdirSummary::Mail(_) | PimdirSummary::Contact(_) => None,
    }
}

/// What queueing an intent wrote.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PimdirQueued {
    /// The queue row, which `pimdir` operator tooling cancels by.
    pub row: i64,
    /// The public id of the item the intent addresses.
    pub seq: i64,
    /// The source named to perform it, `None` in a store whose sources
    /// declare nothing.
    pub source: Option<String>,
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
    use io_pimdir::{
        placement::{PimdirLevel, PimdirLinkId},
        summary::calendar::{PimdirEventSummary, PimdirJournalSummary, PimdirTaskSummary},
    };

    use super::*;

    #[test]
    fn the_collection_refusal_points_at_the_sync() {
        let message = unsupported("create");
        assert!(message.contains("offline cache"));
        assert!(message.contains("sync"));
    }

    /// One calendar may hold two resources carrying one `UID` (pimdir
    /// SPEC 9), which the store keys apart under their own public ids.
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
        assert_eq!(events[0].summary, "Pre demo woonies");
        assert_eq!(events[1].id, "2");
        assert_eq!(events[1].summary, "Pre demo MINIS");
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
            &undownloaded(),
            &event("20260814T090000Z"),
            &range
        ));
        assert!(!in_range(
            &undownloaded(),
            &event("20260914T090000Z"),
            &range
        ));
        assert!(in_range(&undownloaded(), &task("20260814"), &range));
        assert!(!in_range(&undownloaded(), &dateless, &range));
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

    /// A store holding one calendar, `cal`, carrying `bodies` as items a
    /// sync applied, and the backend over it with their public ids.
    fn store(bodies: &[&str]) -> (tempfile::TempDir, PimdirBackend, Vec<String>) {
        let dir = tempfile::tempdir().unwrap();
        let store = io_pimdir::client::PimdirStore::open(dir.path()).unwrap();
        store.ensure_collection("cal", CALENDAR_KIND).unwrap();
        drop(store);

        let config = || PimdirConfig {
            root: dir.path().to_path_buf(),
            account: None,
        };

        let mut backend = PimdirBackend::new(config()).unwrap();
        for body in bodies {
            backend
                .create_item("cal", body.as_bytes().to_vec())
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
                "seq": queued_row.seq,
                "partstat": "ACCEPTED",
                "comment": "See you there",
            })
        );
        assert_eq!(queued_row.seq.to_string(), ids[0]);
        assert_eq!(queued_row.source, None);
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
                "seq": ids[0].parse::<i64>().unwrap(),
                "comment": "Moved to next week",
            })
        );
    }

    #[test]
    fn an_event_with_nobody_to_answer_or_notify_queues_nothing() {
        let (_dir, mut backend, ids) = store(&[ALONE]);

        let reply = backend.reply("cal", &ids[0], "ACCEPTED", None, None, None);
        assert!(reply.unwrap_err().to_string().contains("ORGANIZER"));

        let cancel = backend.cancel("cal", &ids[0], None, None, None);
        assert!(cancel.unwrap_err().to_string().contains("event delete"));

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

    /// A read and a listing report the one version, the body's hash,
    /// and a staged update moves it at once.
    #[test]
    fn the_version_is_the_body_hash_and_follows_a_staged_update() {
        let (_dir, mut backend, ids) = store(&[ALONE]);

        let read = backend.get_item("cal", &ids[0]).unwrap();
        let listed = backend
            .list_items("cal", CalendarItemQuery::default())
            .unwrap();
        let etag = read.etag.clone().unwrap();
        assert_eq!(listed[0].etag.as_deref(), Some(etag.as_str()));
        assert_eq!(etag, backend.client.reader.hash(ALONE.as_bytes()).0);

        let edited = ALONE.replace("SUMMARY:", "SUMMARY:Edited ");
        backend
            .update_item("cal", &ids[0], edited.clone().into_bytes(), Some(&etag))
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
        use crate::error::{ErrorCode, code_of};

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

        let update = backend.update_item("cal", &ids[0], ALONE.into(), Some("stale"));
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

        assert_eq!(queued_row.source.as_deref(), Some("caldav"));
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
            .unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, ids[0]);
        assert_eq!(
            Event::occurrences(&items[0], Some(&range))[0]
                .starts_at
                .as_deref(),
            Some("2026-10-12T09:00:00+00:00")
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

        assert!(in_range(&undownloaded(), &series(None), &range));
        assert!(in_range(
            &undownloaded(),
            &series(Some("20261005T000000Z")),
            &range
        ));
        assert!(!in_range(
            &undownloaded(),
            &series(Some("20250101T000000Z")),
            &range
        ));
    }
}
