---
cairn: tasks
change: a-stale-pimdir-edit-is-parked
---

# Tasks

- [x] src/pimdir/backend.rs `update_item`: `if_match` on the queued `update` is the item's body hash when the caller's `if_match` passed `check_version`, else `None`.
- [x] Test: an edit queued over the version it read, the event then rewritten by a sync-style owner write, is parked by the drain, not applied, the event keeping the other body, and `pimdir status` shows the row; it failed on the old code.
- [x] CHANGELOG Fixed entry.
- [x] Fold the delta into [backends](../../spec/backends.md); log; land.
