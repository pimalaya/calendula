//! pimdir adapter for the shared cross-protocol client.
//!
//! Reads project the store's items through [`io_pimdir`]'s reader plus
//! the blob store. An item whose body is not local still lists, carrying
//! no bytes; only a read of that item reports "body not fetched", which
//! is the cue to sync rather than a data-loss error.
//!
//! Writes append one action to the store's queue (pimdir SPEC 15.1)
//! through a producer opened for that write: the body reaches the blob
//! tree first, then the row pinning it. The store's owner, a sync,
//! applies the action and pushes it. The same reader folds the pending
//! queue over its reads, so a staged change shows here before that
//! happens.
//!
//! Calendars themselves come from the sync, so the collection verbs
//! (create, update, delete) are not served here: a cache does not invent
//! collections its source does not have.
//!
//! Ids are the store's public `seq`, a small integer stable across every
//! collection an item is filed in, never the internal link id.

use std::{io::Write, path::PathBuf};

use anyhow::{Result, anyhow, bail};
use chrono::{SecondsFormat, Utc};
use io_pimdir::{
    PimdirCollection, PimdirItem,
    codec::PimdirAction,
    conventions::{PimdirDerivation, calendar::PimdirCalendarMeta},
};
use io_replica::{object::ReplicaHash, placement::ReplicaFlags};
use log::warn;
use pimalaya_cli::printer::Printer;
use pimalaya_config::toml::TomlConfig;

use crate::{
    cli::load_config,
    config::PimdirConfig,
    pimdir::{
        client::PimdirClient,
        status::{PimdirCalendarStatus, PimdirStatus},
    },
    shared::{
        calendar::{Calendar, CalendarDiff},
        client::paginate,
        event::Event,
        item::{CalendarItem, CalendarTimeRange},
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
    /// the store. Bails when the account carries no `[pimdir]` block.
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

    /// Lists the calendar collections: those declaring `text/calendar`,
    /// plus the kind-less ones a sync created before any consumer
    /// declared a kind.
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

    /// Refuses to create a calendar: declaring a collection is an owner
    /// write (pimdir SPEC 8) and this backend is a producer, and a
    /// collection no sync knows about is one no sync would carry.
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

    /// Lists a collection's items, reading each local body so the
    /// shared type carries the bytes every other backend carries.
    ///
    /// An item that is not hydrated lists with empty contents. A range
    /// filter still applies to it, read off the stored summary rather
    /// than off bytes that are not local yet: an offline cache that
    /// hid its own undownloaded items from a date window would answer
    /// a different question than the one asked.
    pub fn list_items(
        &mut self,
        calendar_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
        range: Option<&CalendarTimeRange>,
    ) -> Result<Vec<CalendarItem>> {
        self.known_collection(calendar_id)?;

        let mut items = Vec::new();

        for stored in self.scan_items(calendar_id)? {
            let item = self.item_from(calendar_id, &stored)?;

            if let Some(range) = range
                && !in_range(&item, &stored, range)
            {
                continue;
            }

            items.push(item);
        }

        Ok(paginate(items, page, page_size))
    }

    /// Reads one item's bytes from its content-addressed blob.
    ///
    /// Fails with a clear "body not fetched" when the item is not
    /// hydrated: that is a state to resolve with a sync, not a missing
    /// item.
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

    /// Stages a locally-authored item as an `add` action the next sync
    /// applies and uploads.
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

    /// Stages a body replacement as an `update` action the next sync
    /// applies and pushes, three-way merging against the stored base.
    ///
    /// `if_match` is ignored: the applied edit is reconciled by the
    /// engine against the base body it recorded at sync time, which is
    /// a stronger guarantee than an entity tag a local store cannot
    /// check.
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

    /// Stages a `remove` action, which the next sync applies as a
    /// tombstone and pushes as a server-side delete.
    pub fn delete_item(&mut self, calendar_id: &str, item_id: &str) -> Result<()> {
        self.known_collection(calendar_id)?;

        let seq = self.item(calendar_id, item_id)?.seq;
        self.enqueue(calendar_id, &PimdirAction::Remove { seq }, None)
    }

    /// Collects the store's accounts and per-calendar hydration state,
    /// for the `pimdir status` command.
    pub fn status(&mut self) -> Result<PimdirStatus> {
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

        Ok(PimdirStatus {
            account: self.client.account.clone(),
            accounts,
            calendars,
        })
    }

    /// The store's calendar collections, narrowed to the configured
    /// account when the store groups several (pimdir SPEC 9.2).
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

    /// Fails unless `calendar_id` names a calendar the store knows,
    /// naming the ones it does hold.
    ///
    /// The store's read seam answers an unknown collection with an
    /// empty page and its queue accepts an action for any name, so
    /// without this a typo in `-k` would read as an empty calendar and
    /// stage into one nothing will ever apply. A calendar is its
    /// collection id, which carries the sync engine's namespace and is
    /// not guessable, so the refusal shows the ids to choose from.
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

    /// Pulls every live item of a collection by keyset paging, in the
    /// order the store maintains for calendars (start ascending).
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

    /// Projects a stored item onto the shared type, reading its body
    /// when one is local and leaving the contents empty otherwise.
    ///
    /// Empty contents mean two different things, and only one of them
    /// is ordinary: an item the sync has not hydrated yet carries no
    /// object at all, while an item naming an object whose blob is gone
    /// is an inconsistent store. The second is logged, since the row
    /// renders the same either way and [`get_item`](Self::get_item)
    /// refuses it outright.
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

    /// Writes a body into the blob tree under the store's own hash,
    /// returning that hash and the committed byte size.
    ///
    /// Durable before anything references it (pimdir SPEC 14), so the
    /// queue row appended next pins a body that is already there.
    fn stage_body(&self, contents: &[u8]) -> Result<(ReplicaHash, u64)> {
        // NOTE: the hash is the store's, read from `store_meta.hash_algo`,
        // never one this crate picks: a body named under another
        // algorithm is a body no read ever finds.
        let hash = self.client.reader.hash(contents);
        let mut writer = self.client.blobs.writer()?;
        writer.write_all(contents)?;
        let size = writer.commit(&hash)?;

        Ok((hash, size))
    }

    /// Appends one action to a collection's queue through a producer
    /// opened for this write and dropped with it.
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

/// The item's link id and `v: 1` summary, as the format derives them
/// (pimdir SPEC Annex A.3), which is what keeps an item staged here and
/// the same item arriving through a sync one item rather than two.
///
/// A queued action carries no sort key: the format leaves the key to the
/// sync that pushes the create.
fn derive(contents: &[u8]) -> PimdirDerivation {
    io_pimdir::conventions::calendar::derive(contents)
}

/// The enqueue timestamp, RFC 3339 as the queue column expects.
fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// Whether an item falls inside `range`.
///
/// A hydrated item is answered from its own bytes, which is exact. An
/// item with no local body falls back to the DTSTART the store's
/// summary carries, which is the whole point of keeping a summary
/// beside the pointer: a cache can answer a date question without the
/// content behind it.
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

/// Reads a stored item's `v: 1` summary, falling back to an empty one
/// when the item was never projected or was written to a shape this
/// version cannot read. A listing showing blank columns beats one that
/// fails.
fn summary_of(item: &PimdirItem) -> PimdirCalendarMeta {
    item.meta
        .as_ref()
        .and_then(|meta| serde_json::from_str(&meta.0).ok())
        .unwrap_or_default()
}

/// Folds a summary stamp into the leading `YYYYMMDD` the range
/// comparison reads, so a summary written by any connector answers the
/// same question as parsed bytes. The digits lead in both an iCalendar
/// value and an RFC 3339 one, so either folds.
fn stamp_of(rfc3339: &str) -> String {
    rfc3339
        .chars()
        .filter(char::is_ascii_digit)
        .take(8)
        .collect()
}

/// The message a collection verb refuses with, naming what to do
/// instead.
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

    /// An added item links the way the store spells it: the bare `UID`
    /// pimdir SPEC Annex A.3 gives, which is what a synced copy carries,
    /// so a staged add naming an identity the collection already holds
    /// parks (pimdir SPEC 15.3) instead of being filed under a key its
    /// producer never asked for. Minting is the store's answer to what a
    /// source hands over; parking is its answer to a producer authoring
    /// an item the collection already holds.
    #[test]
    fn an_added_item_links_the_way_the_store_spells_it() {
        let raw = b"BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:party@example.org\r\n\
                    SUMMARY:Party\r\nDTSTART:20260814T090000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let derived = derive(raw);

        assert_eq!(derived.link_id.0, "party@example.org");
        assert!(derived.meta.0.contains("\"v\":1"));
        assert!(derived.meta.0.contains("Party"));
    }

    /// One calendar may hold two resources whose bodies carry one `UID`
    /// (pimdir SPEC 9): the store keys them apart and draws each its own
    /// public id, so both list as ordinary items. What addresses an item
    /// here is that id, never the identity its body states, and the two
    /// copies need not even be the same event.
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

        // The identity the two bodies state is one string, so it names
        // both of them and addresses neither.
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
