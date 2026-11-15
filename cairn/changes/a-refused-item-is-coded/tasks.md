---
cairn: tasks
change: a-refused-item-is-coded
---

# Tasks

- [x] `ErrorCode::ItemInvalid` (src/error.rs).
- [x] CalDAV: `rejects_item` and its arm in `status_code` (src/caldav/client.rs); `create_error`, `update_error` and `invalid_item`, the hint (src/caldav/backend.rs).
- [x] Google: `invalid_event` (src/gcal/client.rs) on `events.insert`, `events.import` and `events.update` (src/gcal/backend.rs).
- [x] Tests: src/caldav/backend.rs `a_refused_write_carries_its_code` (a 403 `valid-calendar-data` read `None` before) and `a_refused_icalendar_is_invalid_and_keeps_the_answer_as_its_cause` (four preconditions, a 400 naming one, two plain bodies; create and update; `max-resource-size` uncoded); src/gcal/backend.rs `an_event_write_answered_400_is_invalid` (import, insert and update coded, a 404 create and a 400 listing not). Each code read `None` (or `permission-denied` for the plain 403) on the code before.
- [x] CHANGELOG (Added), MIGRATION.md (the code list).
- [x] Fold the delta into [backends](../../spec/backends.md) and [commands](../../spec/commands.md); log.
