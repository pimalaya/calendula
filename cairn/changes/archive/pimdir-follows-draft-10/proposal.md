---
cairn: change
id: pimdir-follows-draft-10
status: landed
created: 2026-11-13
---

# The pimdir backend follows pimdir draft-10

## Why

io-pimdir 0.11.0 (crates.io; 49e1fd5 plus the version bump) follows pimdir ae8b231, tagged storage-draft-10, sync-draft-10 and search-draft-10. neverest, which owns the store calendula reads and stages into, moves to the same drafts, and a draft's store opens under that draft alone (pimdir STORAGE §4.2), so calendula has to read the format it writes. The release breaks four calls calendula makes and changes what three of its outputs mean.

## What

- `io-pimdir = { version = "0.11", ... }` from crates.io, no patch.
- The enqueue answers the row's handle and the partial supports its gate computed (`PimdirEnqueued`), and `enqueue_with_body_checked` is `enqueue_with_body`: calendula enqueues once and logs the partials the enqueue answers, where a `remove` and an intent ran the gate apart (`PimdirProducer::check`) and then enqueued, two gates per action. The queue row calendula prints stays the integer `handle.id`.
- A `remove` carries `if_match` and `resolves` (STORAGE §15.3). `delete_item` states as `if_match` the body a delete gated on a version read, as `update_item` does, so the owner parks a delete whose item holds another body by the time it drains; ungated, it states none and deletes whatever the item holds. It states no `resolves`, calendula showing and deciding no conflict: the owner parks an `update` or a `remove` meeting one, which the spec now says.
- The coverage of `pimdir search` and `pimdir count` (SEARCH §8.2) gains the flat members `missing` and `unparseable`, the bodies the index could not read or parse, and `zoneDiffers`. `complete` is io-pimdir's `is_complete`, the lag included, so calendula adds no `behind == 0` of its own; it is false over a `missing` or an `unparseable` body only while a term of the query reads a calendar body, and the "Incomplete:" line says those counts on the same terms. The zone line prints when the coverage reports a differing zone, that is for a query holding a date term, where calendula compared the zones itself for every query.
- `lock-replaced`, which a producer raises before its enqueue when the store's lock file was removed or replaced (STORAGE §8), carries its code.
- A store or an index an earlier draft wrote is refused as `store-version` or `index-version` through the existing messages; the version tests take draft-09 as the earlier draft and draft-11 as the newer one.

## Out

The audit lines of pimdir cairn/changes/audit-2026-11-12 that concern calendula and wait: C1 (the `store-version` wording at v1), C2 (stripping `X-PIMDIR-` properties from a resource the user did not write, an owner decision first), C4 (a handle of the store id, incarnation and row in the receipts), the rest of C5 (the account and scoping rules hoisted into io-pimdir). No command resolves a conflict.
