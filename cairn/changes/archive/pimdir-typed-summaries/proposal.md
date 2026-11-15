---
cairn: change
id: pimdir-typed-summaries
status: landed
created: 2026-09-03
---

# pimdir: the store's summary is typed, and the owner derives it

io-replica is retired: its sync engine now lives inside io-pimdir, which implements both the storage and the sync parts of the pimdir standard. Every `Replica*` type is a `Pimdir*` one, the client handles sit under `io_pimdir::client::{reader, producer, blobs}` with no crate-root re-export, and `PimdirProducer::enqueue` takes no timestamp, SQLite stamping the row.

The summary changed shape with it. The `v: 1` JSON blob in `items.meta` and the `io_pimdir::conventions::calendar` derivation behind it are gone. A summary is now a typed row in the event, task or journal table (pimdir STORAGE Annex A.3 to A.5), `PimdirSummary::{Event, Task, Journal}` carrying `PimdirTime` values verbatim with their `TZID` and value type, and `summary::calendar::derive` yields the link id, that row and the sort key from a body, `VTIMEZONE` resolution included. A queued `add` or `update` carries no summary any more: the owner derives it from the body when it applies the action.

calendula's pimdir backend read the JSON summary back through `serde_json` into a local `PimdirCalendarMeta`, folded its stamps by hand, and attached a summary to every action it staged. All of that is now the library's, or the owner's.

## What changes

- Cargo: the `pimdir` feature drops `io-replica`, and io-pimdir comes through a `[patch.crates-io]` path until it is released.
- The backend imports the `Pimdir*` types from their modules, stages `add` and `update` with the body alone, and enqueues without a timestamp.
- A listing scans through `list_summaries`, so each row arrives with its typed summary joined, and an undownloaded item answers its kind from the summary's variant and its date from `dtstart`, then `due` for a task, both verbatim.
- The summary reading, the stamp folding and the derivation wrapper are deleted, along with the chrono clock the enqueue no longer needs.
- The tests build `PimdirItem` rows with typed summaries and cover the kind answer as well as the window.

## What does not change

Ids are the public `seq`, the reader takes no lock and folds the pending queue, a write is one queue action behind a durable blob, the collection verbs refuse, and an unknown calendar fails by name. A store written by an earlier io-pimdir draft is refused by the library with `PimdirError::Stale`, and calendula surfaces that message as it does every open failure: there is no migration and no shim.
