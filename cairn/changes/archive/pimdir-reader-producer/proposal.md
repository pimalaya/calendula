---
cairn: change
id: pimdir-reader-producer
status: landed
created: 2026-08-28
---

# pimdir: calendula holds the one handle it must not hold

io-pimdir 0.3 names the three roles the format defines (pimdir SPEC 8) and gives each one a handle.

`PimdirStore` is the **owner**: it drains the queue, sweeps the objects, purges the trash, and takes an exclusive advisory lock on the store directory for its whole lifetime, a second owner getting `PimdirError::Owned` immediately.

`PimdirReader` is the **read** surface, which takes no lock and carries no write at all. `PimdirProducer` is the **enqueue-only** handle for a process that originates mutations without owning the store.

calendula opens the owner. It reads a store a sync populates and it stages a handful of item mutations, which is a reader plus a producer exactly, and instead it holds the handle that can destroy the store and locks the sync out while it prints an agenda.

Since the lock landed, `calendula event list` run while a sync is in flight fails outright, and a sync started while calendula is running fails the same way.

Two defects sit underneath:

**The derivations are calendula's own.** src/pimdir/meta.rs is 533 lines of iCalendar scanning that fixes the link id, the `v: 1` summary and the sort key by hand.

io-pimdir now ships those derivations for `text/calendar` (SPEC Annex A.3), and they are the ones a sync writes.

Two writers of one collection disagreeing about the id of a resource carrying no `UID` link it twice and store its body twice, so the two must agree byte for byte, not merely resemble each other.

**The content hash is not FNV-1a.** src/pimdir/hash.rs computes a 128-bit FNV-1a variant rendered as 32 hex chars.

A store names its bodies by the algorithm recorded in `store_meta.hash_algo`, which is `blake3` in lowercase base32. A writer hashing by hand under its own algorithm writes a body no read ever finds.

## What changes

- The client holds a `PimdirReader` built `with_pending`, taking no lock. Reads run beside a sync, and an action calendula staged shows in its own listing before the owner applies it.
- A write opens a `PimdirProducer` for the length of that one write and drops it: the body goes to the blob tree through `PimdirBlobs::writer`, then one queue row.

  `create_item` enqueues `add`, `update_item` `update`, `delete_item` `remove`, each addressing the item by the public `seq` that is already calendula's item id.
- `pimdir.source` goes away and `pimdir.account` takes its place. A queued action is attributed to a producer name, not to a sync source, so there is nothing left to configure or auto-detect.

  The class of failure the auto-detection existed to soften (a write staged against a source no sync drives) cannot arise. What a store several accounts share does need is which account's collections to show, which is the axis pimdir SPEC 9.2 defines.
- The derivations delegate to `io_pimdir::conventions::calendar`, and src/pimdir/meta.rs and src/pimdir/hash.rs go with them. The body hash comes from the store.
- `create_item` reports the item's link id rather than a `seq`, a queued create having no public id until the owner applies it.
- A listing scans in the store's calendar order (`list_items_page_asc`, the sort key ascending) rather than by link id, which is an arbitrary order for a calendar.
- An unknown calendar id fails rather than reading as an empty calendar and staging into a collection nothing will ever apply.
- `pimdir status` reports the account being read, the accounts the store groups, and the queued creations, in place of the source and the synced sources it can no longer see.

## What does not change

The read semantics the cache requirements pin: an item whose body is not local still lists carrying no bytes, `get_item` still refuses it by name, a range filter still answers from the stored summary.

Ids are still the public `seq`, and the store path is still shell-expanded before anything opens it.
