//! # Pimdir backend
//!
//! The shared-API adapter over a pimdir store, projecting its items
//! through [`io_pimdir`]'s reader plus the blob store.
//!
//! An item whose body is not local still lists, carrying no bytes; only
//! a read of it reports "body not fetched", the cue to sync rather than
//! a data-loss error.
//!
//! Writes append one action to the store's queue (pimdir SPEC 15.1):
//! the body reaches the blob tree first, then the row pinning it. The
//! owner, a sync, applies and pushes it, while the reader folds the
//! pending queue over its reads so a staged change shows before that.
//!
//! Calendars come from the sync, so the collection verbs (create,
//! update, delete) are not served here: a cache does not invent
//! collections its source does not have.
//!
//! Ids are the store's public `seq`, stable across every collection an
//! item is filed in, never the internal link id.

use std::{io::Write, path::PathBuf};

use anyhow::{Result, anyhow, bail};
use chrono::{SecondsFormat, Utc};
use ical::component::IcalComponentKind;
use io_pimdir::{
    PimdirCollection, PimdirItem,
    codec::PimdirAction,
    conventions::{
        PimdirDerivation,
        calendar::{PimdirCalendarMeta, derive as derive_calendar},
    },
};
use io_replica::{object::ReplicaHash, placement::ReplicaFlags};
use log::warn;
use pimalaya_cli::printer::Printer;
use pimalaya_config::toml::TomlConfig;
use serde_json::from_str;

use crate::{
    cli::load_config,
    config::PimdirConfig,
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
}

impl PimdirBackend {
    /// Opens the store at the configured root.
    pub fn new(config: PimdirConfig) -> Result<Self> {
        Ok(Self {
            client: PimdirClient::new(config)?,
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

        Self::new(pimdir_config)
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
            bail!(
                "Item `{item_id}` in calendar `{calendar_id}` is not downloaded yet \
                 (body not fetched); run a sync to hydrate it"
            );
        };

        let contents = self.client.blobs.get(&hash)?.ok_or_else(|| {
            anyhow!("Body blob missing for item `{item_id}` in calendar `{calendar_id}`")
        })?;

        Ok(CalendarItem {
            id: stored.seq.to_string(),
            calendar_id: calendar_id.to_owned(),
            etag: None,
            contents,
        })
    }

    /// Stages a locally-authored item as an `add` the next sync pushes.
    ///
    /// Returns the item's link id, its `UID`: a queued create carries
    /// no public `seq` until the store's owner applies it, so there is
    /// no store-assigned id to report yet.
    pub fn create_item(&mut self, calendar_id: &str, contents: Vec<u8>) -> Result<String> {
        self.known_collection(calendar_id)?;

        let derived = derive(&contents);
        let (hash, size) = self.stage_body(&contents)?;

        let action = PimdirAction::Add {
            link_id: Some(derived.link_id.clone()),
            flags: ReplicaFlags::default(),
            object: Some(hash),
            meta: Some(derived.meta),
            handle: None,
        };
        self.enqueue(calendar_id, &action, Some(size))?;

        Ok(derived.link_id.0)
    }

    /// Stages a body replacement as an `update` the next sync pushes.
    ///
    /// The engine three-way merges it against the base body it recorded
    /// at sync time, which is why `if_match` is ignored: that base is a
    /// stronger guarantee than an entity tag a local store cannot check.
    pub fn update_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        contents: Vec<u8>,
        _if_match: Option<&str>,
    ) -> Result<()> {
        self.known_collection(calendar_id)?;

        let seq = self.item(calendar_id, item_id)?.seq;
        let derived = derive(&contents);
        let (hash, size) = self.stage_body(&contents)?;

        let action = PimdirAction::Update {
            seq,
            object: hash,
            meta: Some(derived.meta),
        };
        self.enqueue(calendar_id, &action, Some(size))
    }

    /// Stages a `remove` action: a tombstone, then a server-side delete.
    pub fn delete_item(&mut self, calendar_id: &str, item_id: &str) -> Result<()> {
        self.known_collection(calendar_id)?;

        let seq = self.item(calendar_id, item_id)?.seq;
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

    /// Pulls every live item of a collection by keyset paging.
    ///
    /// In the order the store maintains for calendars, start ascending.
    fn scan_items(&self, calendar_id: &str) -> Result<Vec<PimdirItem>> {
        let mut all = Vec::new();
        let mut cursor: Option<(String, i64)> = None;

        loop {
            let page = self.client.reader.list_items_page_asc(
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
            etag: None,
            contents,
        })
    }

    /// Writes a body into the blob tree, returning its hash and size.
    ///
    /// Durable before anything references it (pimdir SPEC 14), so the
    /// queue row appended next pins a body that is already there.
    fn stage_body(&self, contents: &[u8]) -> Result<(ReplicaHash, u64)> {
        // NOTE: the hash is the store's, from `store_meta.hash_algo`,
        // never one this crate picks: a body named under another
        // algorithm is one no read ever finds.
        let hash = self.client.reader.hash(contents);
        let mut writer = self.client.blobs.writer()?;
        writer.write_all(contents)?;
        let size = writer.commit(&hash)?;

        Ok((hash, size))
    }

    /// Appends one action to a collection's queue.
    ///
    /// The producer is opened for this write and dropped with it.
    fn enqueue(
        &self,
        calendar_id: &str,
        action: &PimdirAction,
        object_size: Option<u64>,
    ) -> Result<()> {
        self.client
            .producer()?
            .enqueue(calendar_id, action, object_size, &now())
            .map_err(|err| anyhow!("Stage the pimdir action: {err}"))?;

        Ok(())
    }
}

/// The item's link id and `v: 1` summary (pimdir SPEC Annex A.3).
///
/// Deriving them the format's way is what keeps an item staged here and
/// the same item arriving through a sync one item rather than two. A
/// queued action carries no sort key: the format leaves it to the sync.
fn derive(contents: &[u8]) -> PimdirDerivation {
    derive_calendar(contents)
}

/// The enqueue timestamp, RFC 3339 as the queue column expects.
fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// Whether an item falls inside `range`.
///
/// A hydrated item is answered from its own bytes, which is exact. One
/// with no local body falls back to the DTSTART its summary carries,
/// the whole point of a summary beside the pointer.
/// Whether a stored item is of `kind`.
///
/// A body the sync has not downloaded still answers, the `v: 1` summary
/// naming the component a reader renders the resource as (pimdir SPEC
/// Annex A.3), so an availability-aware listing stays one.
fn is_kind(item: &CalendarItem, stored: &PimdirItem, kind: IcalComponentKind) -> bool {
    if !item.contents.is_empty() {
        return holds_kind(&item.contents, kind);
    }

    summary_of(stored)
        .component
        .as_deref()
        .is_some_and(|component| component.eq_ignore_ascii_case(&kind))
}

fn in_range(item: &CalendarItem, stored: &PimdirItem, range: &CalendarTimeRange) -> bool {
    if !item.contents.is_empty() {
        return Event::project(item)
            .iter()
            .any(|event| range.contains(&event.start));
    }

    let meta = summary_of(stored);

    // NOTE: DTSTART then DUE, the same order the sort key takes, so a
    // to-do carrying only a due date still answers a date question.
    meta.dtstart
        .as_deref()
        .or(meta.due.as_deref())
        .map(|start| range.contains(&stamp_of(start)))
        .unwrap_or(false)
}

/// Reads a stored item's `v: 1` summary, or an empty one.
///
/// An item never projected, or written to a shape this version cannot
/// read, falls back: blank columns beat a listing that fails.
fn summary_of(item: &PimdirItem) -> PimdirCalendarMeta {
    item.meta
        .as_ref()
        .and_then(|meta| from_str(&meta.0).ok())
        .unwrap_or_default()
}

/// Folds a summary stamp into the leading `YYYYMMDD` a range compares.
///
/// The digits lead in both an iCalendar value and an RFC 3339 one, so
/// a summary written by any connector answers as parsed bytes do.
fn stamp_of(rfc3339: &str) -> String {
    rfc3339
        .chars()
        .filter(char::is_ascii_digit)
        .take(8)
        .collect()
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
    use io_replica::placement::{ReplicaLevel, ReplicaLinkId, ReplicaMeta};

    use super::*;

    #[test]
    fn the_collection_refusal_points_at_the_sync() {
        let message = unsupported("create");
        assert!(message.contains("offline cache"));
        assert!(message.contains("sync"));
    }

    #[test]
    fn a_summary_stamp_folds_onto_the_day_the_range_compares() {
        assert_eq!(stamp_of("2026-08-14T09:00:00Z"), "20260814");
        assert_eq!(stamp_of(""), "");
    }

    /// A staged add links under the bare `UID` (pimdir SPEC Annex A.3),
    /// which is what a synced copy carries.
    ///
    /// So an add naming an identity the collection already holds parks
    /// (pimdir SPEC 15.3) instead of being filed under a key its
    /// producer never asked for.
    #[test]
    fn an_added_item_links_the_way_the_store_spells_it() {
        let raw = b"BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:party@example.org\r\n\
                    SUMMARY:Party\r\nDTSTART:20260814T090000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let derived = derive(raw);

        assert_eq!(derived.link_id.0, "party@example.org");
        assert!(derived.meta.0.contains("\"v\":1"));
        assert!(derived.meta.0.contains("Party"));
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
            derive(&woonies.contents).link_id.0,
            derive(&minis.contents).link_id.0
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
        let item = CalendarItem {
            id: "1".into(),
            calendar_id: "personal".into(),
            etag: None,
            contents: Vec::new(),
        };
        let stored = |start: &str| PimdirItem {
            seq: 1,
            link_id: ReplicaLinkId("party@example.org".into()),
            flags: ReplicaFlags::default(),
            meta: Some(ReplicaMeta(format!(
                "{{\"v\":1,\"summary\":\"x\",\"dtstart\":\"{start}\"}}"
            ))),
            sort_key: String::new(),
            object: None,
            level: ReplicaLevel::Meta,
            retention: None,
        };

        assert!(in_range(&item, &stored("20260814T090000Z"), &range));
        assert!(!in_range(&item, &stored("20260914T090000Z"), &range));
    }
}
