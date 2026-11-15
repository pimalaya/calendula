---
cairn: tasks
change: a-create-refuses-a-held-uid
---

# Tasks

- [x] Tests: src/gcal/backend.rs `a_create_naming_a_held_uid_is_refused_and_imports_nothing` (an empty first page with a token, then a page holding the UID: `item-duplicate`, two GETs, no POST); src/caldav/backend.rs `a_duplicate_uid_and_a_stale_version_carry_their_code` gains the 412 to a create. Both failed on the code before (the import answered, the 412 `precondition-failed`).
- [x] `GcalBackend::holding` and the check in `create_item` (src/gcal/backend.rs); the stash test answers the lookup first.
- [x] The 412 arm of `CaldavBackend::create_item` (src/caldav/backend.rs).
- [x] CHANGELOG (Changed); MIGRATION.md ("Commands that now fail earlier").
- [x] Fold the delta into [backends](../../../spec/backends.md) and [commands](../../../spec/commands.md); log.
