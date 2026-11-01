---
cairn: tasks
change: an-online-meeting-ask-is-gated-by-the-item-sources
---

# Tasks

- [x] Test: in a calendar a declaring and an undeclared source sync, an update asking for a meeting on an item only the undeclared one binds queues nothing; fails on the collection read.
- [x] `update_item` checks the item's sources.
- [x] CHANGELOG: folded into the Unreleased `--online-meeting` Added entry, the check being new since 0.2.0.
- [x] Fold the delta into [backends](../../spec/backends.md); log; land.
