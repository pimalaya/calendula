---
cairn: log
change: pimdir-honours-if-match
landed: 2026-10-04
---

# pimdir honours `--if-match`, and every event shows its version

The pimdir backend reports the store's hash of an item's body, pending queue folded in, as its `etag`, and its `update_item` and `delete_item` refuse a stale `if_match` before queueing anything, with an error starting `Precondition failed:`. The event JSON carries its item's `etag` on every backend, `item read` reports it, and the four delete commands take `--if-match` (CalDAV `If-Match`, msgraph `changeKey`, refused on vdir and gcal).

Capabilities moved: **backends** (a pimdir item's version is its body hash; pimdir writes are staged queue actions, modified) and **commands** (a delete can be gated on a version; the event projection, modified).
