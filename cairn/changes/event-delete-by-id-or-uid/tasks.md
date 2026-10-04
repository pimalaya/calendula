---
cairn: tasks
change: event-delete-by-id-or-uid
---

# Tasks

- [x] Resolve the `event delete` argument as an id, then as a `UID`; refuse an ambiguous `UID`.
- [x] Reword the `item`, `todo` and `journal delete` (and `item read`) argument docs.
- [x] Tests: an id wins over a `UID`, a `UID` names its one item, a shared `UID` is refused, a miss is clear.
- [x] Fold the delta into [commands](../../spec/commands.md); write the log entry.
