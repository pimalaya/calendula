---
cairn: tasks
change: a-caldav-read-answered-400-is-a-miss
---

# Tasks

- [x] Test: src/caldav/backend.rs `a_read_answered_400_is_not_found_and_a_write_is_not`, a scripted server answering 400 to `GET <cal>/nope` (`item-not-found`, `Read item ...`, `HTTP 400` in the chain) and to a PUT (no code). The read failed with no code on the code before.
- [x] The 400 arm of `CaldavBackend::get_item` (src/caldav/backend.rs).
- [x] CHANGELOG (Changed).
- [x] Fold the delta into [backends](../../../spec/backends.md); log.
