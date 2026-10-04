---
cairn: tasks
change: pimdir-honours-if-match
---

# Tasks

- [x] pimdir: report the body hash as `etag` on reads and listings; check `if_match` in `update_item` and `delete_item`.
- [x] Shared: `etag` on the event projection and in `item read`; `if_match` on `delete_item` and `--if-match` on the four delete commands; CalDAV, msgraph, vdir and gcal adapters.
- [x] Tests: the version follows a staged update; a stale tag refuses update and delete, queueing nothing.
- [x] Fold the delta into [backends](../../spec/backends.md) and [commands](../../spec/commands.md); write the log entry.
