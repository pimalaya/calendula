---
cairn: change
change: pimdir-reader-producer
---

# Delta

## ADDED Requirements

### Requirement: pimdir takes the reader and producer roles, never the owner
The pimdir backend SHALL read through a `PimdirReader` and write through a `PimdirProducer`, and SHALL NOT open a `PimdirStore`.

The owner handle drains the queue, sweeps the objects and purges the trash, and holds an exclusive lock on the store for its lifetime, so holding it would both lock a sync out for the length of a listing and put every destructive verb behind a frontend that never calls them.

The reader SHALL be built with the pending overlay, so an action this process staged reads back before the store's owner applies it.

### Requirement: A pimdir body is named by the store's own hash
The pimdir backend SHALL name a body it writes with the hash the store records in `store_meta.hash_algo`, read through the handle it holds, and SHALL NOT compute a digest of its own choosing. A body named under the wrong algorithm is a body no read ever finds.

### Requirement: pimdir refuses an unknown calendar
Every pimdir read and write SHALL fail when the calendar id names no collection of the account.

The store's read seam answers an unknown collection with an empty page and its queue accepts an action for any name, so without this a typo in `-k` would read as an empty calendar and stage into one nothing will ever apply.

### Requirement: pimdir lists in the store's calendar order
A pimdir listing SHALL scan the collection by the store's own sort key, ascending, which is the item's resolved start. Paging by link id is an arbitrary order for a calendar, and the store maintains the one a reader expects.

### Requirement: A missing blob is reported, not hidden
An item naming an object whose blob file is absent SHALL still list, and the listing SHALL log a warning naming the item.

The store is inconsistent rather than partially synced, the row renders the same as an unhydrated one, and `get_item` refuses it outright, so a silent blank row hides a broken store the read reports.

## MODIFIED Requirements

### Requirement: pimdir backend
pimdir SHALL adapt io-pimdir over io-replica. The store is an offline cache a sync engine fills, not a server: reads project the store's items and writes are queue actions a later sync applies and propagates.

Collections come from the sync, so `create_calendar`, `update_calendar` and `delete_calendar` SHALL refuse with a message pointing at the account the store syncs.

A collection SHALL be listed as a calendar when it declares `text/calendar`, or when it declares no kind at all (a sync created it before any consumer declared one).

A store grouping its collections under accounts (pimdir SPEC 9.2) SHALL be narrowed to `pimdir.account` when that is set, and read whole when it is not.

### Requirement: pimdir writes are staged queue actions
A pimdir write SHALL append one action to the store's queue (pimdir SPEC 15.1) through a producer opened for that write and dropped after it: `create_item` to `add`, `update_item` to `update`, `delete_item` to `remove`.

The body SHALL reach the blob tree through the blob writer, durably, before the row that pins it is appended, and the action SHALL address the item by the public `seq` that is already the item's shared id.

`update_item` SHALL ignore `--if-match`, because the engine reconciles the applied edit against the base body it recorded at sync time, which is stronger than an entity-tag precondition a local store cannot check.

Because a queued create carries no public id until the owner applies it, `create_item` SHALL report the item's link id instead.

### Requirement: The text/calendar summary convention
The link id, the `v: 1` summary and the sort key a pimdir write records SHALL be derived by `io_pimdir::conventions::calendar`, the format's own derivations (pimdir SPEC Annex A.3), so an item calendula stages links and summarises exactly as the same item arriving through a sync.

The link id is the bare `UID`, with nothing prepended. A queued action carries no sort key: the format leaves the key to the sync that pushes the write, and a producer deriving one would order an item the connector is about to reorder.

calendula SHALL read that summary to answer a date question about an item whose body is not local: `dtstart`, then `due` for a to-do carrying no start.

### Requirement: Backend-specific commands
pimdir covers `status`, reporting the account being read, every account the store groups collections under, how much of each calendar is downloaded, and how many creations are queued for the next sync.

### Requirement: Account configuration
An account's `pimdir` block SHALL carry a `root` and an optional `account`.

## REMOVED Requirements

### Requirement: pimdir writes are staged and source-guarded
Removed: replaced by the queue-action requirement above. A staged action is attributed to a producer name rather than to a replica source, so there is no source to guard a write against and no placement base to check.

### Requirement: pimdir writes auto-source
Removed: a queued action is attributed to a producer name rather than to a replica source, so there is no source to configure, none to auto-detect from the store, and no store-was-not-synced-as-this-source failure left to report.

`pimdir.source` leaves the configuration with it, replaced by `pimdir.account`.
