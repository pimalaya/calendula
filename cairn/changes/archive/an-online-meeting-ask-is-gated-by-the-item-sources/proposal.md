---
cairn: change
id: an-online-meeting-ask-is-gated-by-the-item-sources
status: landed
created: 2026-10-30
---

# An online-meeting ask on an update is gated by the item's sources

## Why

pimdir STORAGE §15.6 gates an action on an existing item by the sources binding that item, the ones that will push it, and an `add` by the sources syncing the collection. calendula's own online-meeting check read the collection's sources on `update_item` too (src/pimdir/backend.rs:358-362). In a calendar two sources sync, an update asking for a meeting on an item only an undeclared source binds passed because the other source declares `calendar.online-meeting`; the store's gate then let it through (an undeclared source refuses nothing), and the ask was queued for a sync engine that would write it and never act on it, which is exactly what the check exists to refuse.

## What

- `update_item` reads the item's capabilities (`item_capabilities`) for the online-meeting check; `create_item` keeps the collection's.
- Both read through the reader, which also leaves one producer per write (change a-body-is-staged-under-the-lock).
