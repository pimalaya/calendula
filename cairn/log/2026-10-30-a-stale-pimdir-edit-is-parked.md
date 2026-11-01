---
cairn: log
change: a-stale-pimdir-edit-is-parked
landed: 2026-10-30
---

# A stale pimdir edit is parked

`update_item` (src/pimdir/backend.rs) queues its `update` with `if_match`, io-pimdir's new field (pimdir STORAGE §15.3): the item's body hash when the caller's `if_match` passed `check_version`, which the `update` commands set from the item they read or from `--if-match`, and `None` when a replacing source came with no `--if-match`. The owner parks the row when a sync replaced the body while it waited, naming both bodies, and `pimdir status` lists it.

Test: `an_edit_whose_body_changed_underneath_is_parked` queues an edit over the version read, lands another body on the synced event through an owner write, and drains. On the old code (`if_match: None`) it failed with `PimdirDrainReport { applied: 1, parked: 0, .. }`: the edit overwrote the other body.

Capabilities moved: **backends** (a queued pimdir edit names the body it replaced, added).
