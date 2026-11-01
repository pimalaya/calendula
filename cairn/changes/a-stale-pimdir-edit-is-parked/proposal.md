---
cairn: change
id: a-stale-pimdir-edit-is-parked
status: landed
created: 2026-10-30
---

# A stale pimdir edit is parked

## Why

`--if-match` and the version an update reads gate a pimdir edit when it is queued, not when the store's owner applies it. A sync landing the server's own edit of the event in between left the owner applying calendula's edit over it, dropping that change without a merge. io-pimdir (2026-10-30, pimdir STORAGE §15.3) gives `PimdirAction::Update` an `if_match` naming the body the producer read, and parks the row when the item holds another by then.

## What

- `update_item` states, as `if_match` on its `update`, the body it was gated on: the version `--if-match` named, or the one `event`, `todo`, `journal` and `item update` read when they start from the stored item. An update given a replacing source and no `--if-match` read no body, and states none.
- The parked edit surfaces as any parked action does: `pimdir status` lists it by its queue row, with the owner's error naming both bodies.
