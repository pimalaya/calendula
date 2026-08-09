//! pimdir adapter for the shared cross-protocol client.
//!
//! Reads project the store's items through [`io_pimdir`]'s client read
//! API plus the blob reader. An item whose body is not local still
//! lists, carrying no bytes; only a read of that item reports "body not
//! fetched", which is the cue to sync rather than a data-loss error.
//!
//! Writes stage [`io_replica`] mutations through the store's `mutate`
//! seam, never raw SQL, so the next sync derives and pushes them. Each
//! is attributed to the client's configured source and fails loudly
//! when the store was not synced as that source, rather than silently
//! staging a change no sync will carry.
//!
//! Ids are the store's public `seq`, a small integer stable across
//! every collection an item is filed in, never the internal link id.

use std::path::PathBuf;

use anyhow::{Result, anyhow, bail};
use io_pimdir::PimdirItem;
use io_replica::{
    client::ReplicaStorage,
    collection::ReplicaCollectionId,
    coroutine::{ReplicaArg, ReplicaCoroutine, ReplicaCoroutineState, ReplicaYield},
    mutate::{ReplicaMutate, ReplicaMutation},
    object::ReplicaObject,
    placement::{ReplicaFlags, ReplicaHandle, ReplicaPlacement},
};

use pimalaya_config::toml::TomlConfig;

use crate::{
    cli::load_config,
    config::PimdirConfig,
    pimdir::{
        client::PimdirClient,
        hash::content_hash,
        meta::{CALENDAR_KIND, CalendarMeta, project},
        status::{PimdirCalendarStatus, PimdirStatus},
    },
    shared::{
        calendars::{Calendar, CalendarDiff},
        client::paginate,
        events::Event,
        items::{CalendarItem, CalendarTimeRange},
    },
};

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
    pub fn build(config_paths: &[PathBuf], account_name: Option<&str>) -> Result<Self> {
        let mut config = load_config(config_paths)?;
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
            .client
            .store
            .list_collections()?
            .into_iter()
            .filter(|collection| collection.kind.is_empty() || collection.kind == CALENDAR_KIND)
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

    /// Refuses to create a calendar: a cache holds the collections its
    /// sync source has, and inventing one here would produce a calendar
    /// no server knows about and no sync would ever carry.
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
        let seq = parse_id(item_id)?;
        let Some(stored) = self.client.store.get_item(calendar_id, seq)? else {
            bail!("Item `{item_id}` not found in calendar `{calendar_id}`");
        };

        let Some(hash) = stored.object.clone() else {
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

    /// Stages a locally-authored item as an `Add` the next sync
    /// uploads. Returns the public id the store assigned it.
    pub fn create_item(&mut self, calendar_id: &str, contents: Vec<u8>) -> Result<String> {
        let projection = project(&contents);
        let link = projection.link_id.0.clone();
        let object = ReplicaObject {
            hash: content_hash(&contents),
            size: contents.len(),
        };

        self.mutate(
            calendar_id,
            ReplicaMutation::Add {
                handle: ReplicaHandle(format!("local:{link}")),
                link_id: projection.link_id,
                flags: ReplicaFlags::default(),
                object,
                body: contents,
                meta: Some(projection.meta),
                sort_key: projection.sort_key,
            },
        )?;

        let seq = self
            .client
            .store
            .seq_for_link(calendar_id, &link)?
            .ok_or_else(|| anyhow!("Added item `{link}` in `{calendar_id}` has no public id"))?;

        Ok(seq.to_string())
    }

    /// Stages a content change as an `Edit` the next sync pushes.
    ///
    /// pimdir carries no entity tag of its own, so `if_match` cannot be
    /// honoured: the engine's own three-way merge against the stored
    /// base is what guards a concurrent remote change.
    pub fn update_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        contents: Vec<u8>,
        _if_match: Option<&str>,
    ) -> Result<()> {
        let placement = self.synced_placement(calendar_id, item_id)?;
        let projection = project(&contents);
        let object = ReplicaObject {
            hash: content_hash(&contents),
            size: contents.len(),
        };

        self.mutate(
            calendar_id,
            ReplicaMutation::Edit {
                handle: placement.handle,
                object,
                body: contents,
                meta: Some(projection.meta),
                // NOTE: an edit that moves DTSTART has to restate the
                // key, or the item stays sorted where its old start put
                // it.
                sort_key: Some(projection.sort_key),
            },
        )
    }

    /// Stages a removal as a tombstone the next sync pushes.
    pub fn delete_item(&mut self, calendar_id: &str, item_id: &str) -> Result<()> {
        let placement = self.synced_placement(calendar_id, item_id)?;
        self.mutate(calendar_id, ReplicaMutation::Remove(placement.handle))
    }

    /// Collects the store's sources and per-calendar hydration state,
    /// for the `pimdir status` command.
    pub fn status(&mut self) -> Result<PimdirStatus> {
        let sources = self.client.store.distinct_sources()?;
        let mut calendars = Vec::new();

        for collection in self.client.store.list_collections()? {
            if !collection.kind.is_empty() && collection.kind != CALENDAR_KIND {
                continue;
            }

            let items = self.scan_items(&collection.id)?;
            let hydrated = items.iter().filter(|item| item.object.is_some()).count();

            calendars.push(PimdirCalendarStatus {
                name: if collection.name.is_empty() {
                    collection.id.clone()
                } else {
                    collection.name
                },
                id: collection.id,
                total: items.len(),
                hydrated,
            });
        }

        calendars.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(PimdirStatus {
            source: self.client.source().to_owned(),
            sources,
            calendars,
        })
    }

    /// Pulls every live item of a collection by keyset paging: the
    /// store's read API is paginated, and the shared commands sort and
    /// paginate in memory as the other local backend does.
    fn scan_items(&self, calendar_id: &str) -> Result<Vec<PimdirItem>> {
        let mut all = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let page = self
                .client
                .store
                .list_items(calendar_id, cursor.as_deref(), SCAN_BATCH)?;
            let count = page.len();

            if let Some(last) = page.last() {
                cursor = Some(last.link_id.0.clone());
            }

            all.extend(page);

            if count < SCAN_BATCH {
                break;
            }
        }

        Ok(all)
    }

    /// Projects a stored item onto the shared type, reading its body
    /// when one is local and leaving the contents empty otherwise.
    fn item_from(&self, calendar_id: &str, stored: &PimdirItem) -> Result<CalendarItem> {
        let contents = match &stored.object {
            Some(hash) => self.client.blobs.get(hash)?.unwrap_or_default(),
            None => Vec::new(),
        };

        Ok(CalendarItem {
            id: stored.seq.to_string(),
            calendar_id: calendar_id.to_owned(),
            etag: None,
            contents,
        })
    }

    /// The source's placement for the public id `item_id`, guaranteed
    /// to carry a sync base.
    ///
    /// A change staged on a placement with no base would look like a
    /// fresh create rather than an edit, and no sync would carry it, so
    /// this is the guard that turns a misconfigured source into a clear
    /// error instead of a silent no-op.
    fn synced_placement(&self, calendar_id: &str, item_id: &str) -> Result<ReplicaPlacement> {
        let seq = parse_id(item_id)?;
        let link_id = self
            .client
            .store
            .get_item(calendar_id, seq)?
            .map(|item| item.link_id.0)
            .ok_or_else(|| anyhow!("Item `{item_id}` not found in calendar `{calendar_id}`"))?;

        let loaded = self
            .client
            .store
            .load(&ReplicaCollectionId(calendar_id.to_owned()))?;

        let placement = loaded
            .placements
            .into_iter()
            .find(|placement| {
                placement.link_id.as_ref().map(|link| link.0.as_str()) == Some(link_id.as_str())
            })
            .ok_or_else(|| anyhow!("Item `{item_id}` not found in calendar `{calendar_id}`"))?;

        if placement.base.is_none() {
            bail!(
                "Calendar `{calendar_id}` was not synced as source `{}`, so item `{item_id}` \
                 cannot be edited here; set `pimdir.source` to the sync source and sync first",
                self.client.source()
            );
        }

        Ok(placement)
    }

    /// Drives a `mutate` coroutine to completion against the store: it
    /// only ever asks to load the collection and to write the staged
    /// operations.
    fn mutate(&mut self, calendar_id: &str, mutation: ReplicaMutation) -> Result<()> {
        let mut coroutine = ReplicaMutate::new(calendar_id.to_owned(), mutation);
        let mut arg: Option<ReplicaArg> = None;

        loop {
            match coroutine.resume(arg.take()) {
                ReplicaCoroutineState::Yielded(ReplicaYield::WantsLoad(collection)) => {
                    let loaded = self.client.store.load(&collection)?;
                    arg = Some(ReplicaArg::Load(loaded));
                }
                ReplicaCoroutineState::Yielded(ReplicaYield::WantsWrite(ops)) => {
                    self.client.store.write(ops)?;
                    arg = Some(ReplicaArg::Write);
                }
                ReplicaCoroutineState::Yielded(_) => {
                    bail!("pimdir mutate asked for an unexpected step");
                }
                ReplicaCoroutineState::Complete(result) => {
                    return result.map_err(|err| anyhow!("pimdir mutate failed: {err}"));
                }
            }
        }
    }
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

    CalendarMeta::read(stored.meta.as_ref())
        .start
        .as_deref()
        .map(|start| range.contains(&stamp_of(start)))
        .unwrap_or(false)
}

/// Folds an RFC 3339 summary stamp into the leading `YYYYMMDD` the
/// range comparison reads, so a summary written by any connector
/// answers the same question as parsed bytes.
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

/// Parses the public id a listing showed. Anything non-numeric is the
/// internal link id or a mistyped value, and saying so beats a lookup
/// that silently finds nothing.
fn parse_id(id: &str) -> Result<i64> {
    id.parse()
        .map_err(|_| anyhow!("Invalid pimdir item id `{id}`: expected the number a listing showed"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_public_id_parses_and_a_link_id_is_rejected_by_name() {
        assert_eq!(parse_id("42").unwrap(), 42);

        let err = parse_id("uid:event-1@example.org").unwrap_err().to_string();
        assert!(
            err.contains("expected the number a listing showed"),
            "{err}"
        );
    }

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

    #[test]
    fn an_undownloaded_item_is_windowed_from_its_summary() {
        use io_replica::placement::{ReplicaLevel, ReplicaLinkId, ReplicaMeta};

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
            link_id: ReplicaLinkId("uid:x".into()),
            flags: ReplicaFlags::default(),
            meta: Some(ReplicaMeta(format!(
                "{{\"v\":1,\"summary\":\"x\",\"start\":\"{start}\"}}"
            ))),
            object: None,
            level: ReplicaLevel::Meta,
            sort_key: Default::default(),
        };

        assert!(in_range(&item, &stored("2026-08-14T09:00:00Z"), &range));
        assert!(!in_range(&item, &stored("2026-09-14T09:00:00Z"), &range));
    }
}
