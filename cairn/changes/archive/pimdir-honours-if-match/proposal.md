---
cairn: change
id: pimdir-honours-if-match
status: landed
created: 2026-10-04
---

# pimdir honours `--if-match`, and every event shows its version

## Why

A caller that previews a change and writes it later (MOA, asking the user to confirm) has to know the event it writes over is still the one it showed. On pimdir `--if-match` was ignored and no read reported a version, so MOA hashed each event itself to compare. The sync engine's three-way merge protects the store, not the caller's preview.

## What

- The pimdir version of an item is the store's hash of its body, the pending queue folded in. `get_item` and `list_items` report it as `etag`; an item whose body is not local has none.
- The event JSON (`event list`, `event read`, `event find`) carries the `etag` of its item, and `item read` adds `etag` to its JSON.
- pimdir `update_item` and `delete_item` refuse a write whose `if_match` is not that version, before anything is queued, with an error starting `Precondition failed`.
- `event`, `todo`, `journal` and `item delete` take `--if-match`. CalDAV sends it as `If-Match`, msgraph compares the `changeKey`, and vdir and gcal, which cannot gate a delete, refuse it.
