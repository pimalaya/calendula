---
cairn: change
id: a-missing-pimdir-body-is-pending
status: landed
created: 2026-11-18
---

# A missing pimdir body is pending

## Why

Audit line TRI-3 of the CLI family analysis (2026-11-15) and the owner ruling of 2026-11-18. An item whose stored object the blob tree no longer holds (a `pimdir item erase`, a quarantine, a purge) listed with an `item-unreadable` entry and the etag of the body it could not read, the command exiting 3, and its `item read`, `event read`, `pimdir reply` and `pimdir cancel` failed uncoded ("Body blob missing for item ..."). After an erase every listing of the calendar exited 3 until a resync. pimdir's own guide says such a body "is not hydrated and not an error" (pimdir GUIDE §14), and Himalaya lists the row as usual and answers `body-pending` on read. The ruling: the guide and Himalaya win over the twin specs.

## What

- `item list`, `event list`, `event agenda` and the other pimdir listings list such an item as an unfetched one: no bytes, `etag` `null`, no `errors` entry, no exit 3. `pimdir search` listed it so already.
- `item read` and `event read` fail coded `body-pending` with the sentence of an unfetched item, ``Item `<id>` in calendar `<calendar>` is not downloaded yet (body not fetched); run a sync to hydrate it``, exit 1, the failure object Himalaya prints for a message in the same state; `pimdir reply`, `pimdir cancel` and `event update`, which read the body first, refuse with the same code, nothing queued.
- A body that cannot be read (an I/O error) stays an `item-unreadable` entry, now with no `etag` either, as cardamum lists it.
- cardamum takes the same change (its `a-missing-pimdir-body-is-pending`).
