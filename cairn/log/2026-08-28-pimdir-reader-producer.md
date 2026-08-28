---
cairn: log
date: 2026-08-28
change: pimdir-reader-producer
---

# pimdir: the reader and the producer, never the owner

io-pimdir 0.3 gave each of the three roles pimdir SPEC 8 defines a handle of its own, and put an exclusive advisory lock on the owner. calendula was opening the owner to print an agenda, so a listing and a sync could no longer run at the same time, in either order. It also carried its own derivations and its own content hash for a store that names both.

## What landed

**[src/pimdir/client.rs](../../src/pimdir/client.rs) holds a reader and opens producers.** `PimdirReader::open(root).with_pending()` takes no lock and folds the queue's pending actions over the committed rows, so a staged edit or deletion shows on the next read. A write opens a `PimdirProducer` for the length of that write and drops it: the shared lock it takes is what keeps a collector out of the window between a body reaching the blob tree and the queue row pinning it.

**[src/pimdir/backend.rs](../../src/pimdir/backend.rs) enqueues instead of mutating.** `create_item` appends an `add`, `update_item` an `update`, `delete_item` a `remove`, each addressing the item by the public `seq` a listing showed. The body reaches the blob tree through `PimdirBlobs::writer` first, under the hash the store records in `store_meta.hash_algo`. A queued create has no public id until the sync applies it, so `create_item` reports the link id.

**The derivations are the format's.** `src/pimdir/meta.rs` and `src/pimdir/hash.rs` are gone, replaced by `io_pimdir::conventions::calendar` and the store's own hasher. The link id becomes the bare `UID`, which is what the store already holds, so an item calendula stages and the same item arriving through a sync are one item. calendula still reads the `v: 1` summary, to window an item whose body is not local.

**`pimdir.source` became `pimdir.account`.** A queued action is attributed to a producer name, so there was no source left to configure, none to auto-detect, and no store-was-not-synced-as-this-source failure to report. What a shared store does need is which account's collections to show, the axis pimdir SPEC 9.2 defines.

**Two reads got sharper.** A listing pages by the store's sort key ascending, which is calendar order, rather than by link id. An unknown calendar id fails by name instead of reading as an empty calendar and staging into a collection nothing will apply.

**`pimdir status` reports what it can now see:** the account being read, the accounts the store groups, and per calendar the downloaded count plus the queued creations.

Beside it, `CaldavCalendar` gained `supported_reports` in io-webdav, so `caldav calendar create` fills the struct with `..Default::default()`.

## Capabilities moved

- backends: the pimdir backend, its write model, its derivations, its ordering and its unknown-collection behaviour
- commands: `pimdir status`
- config: the `pimdir` account block

## Verified against a live store

A read-only run against the Neverest store at `~/.local/state/neverest/posteo` (454 items in `caldav/default`), reported under [cairn/spec/testing/pimdir-local.md](../spec/testing/pimdir-local.md). It confirmed the two silent bugs the rework fixes: `io_pimdir::conventions::calendar` reproduces the store's stored link id and `v: 1` summary byte for byte, and the store's own hash reproduces a body's stored base32 blake3 name, where the retired FNV-1a digest would have named a body no read finds. A listing run while another process held `owner.lock` succeeded, which the owner handle could not do.

The write paths were then run against a clone of the store's index in a scratch directory: `add`, `update` and `remove` each append one correctly addressed queue row under the producer name `calendula`, and the pending overlay makes a staged edit read back and a staged deletion disappear before any owner applies them. The production store was confirmed untouched afterwards.

Three defects came out of the run. `pimdir status` asserted the store grouped no account while naming one on the next line, and an item naming an object whose blob file is gone listed as a blank row while `get_item` refused it by name, the listing hiding a broken store the read reports; both are fixed here. `event list` returning an empty first page is left open: the listing paginates before it projects, and calendar order now puts this store's keyless VTODOs first, so the fix is a shared-command decision rather than a pimdir one.
