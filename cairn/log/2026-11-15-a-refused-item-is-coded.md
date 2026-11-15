---
cairn: log
change: a-refused-item-is-coded
landed: 2026-11-15
---

# A refused item is coded

An iCalendar a CalDAV server refuses is coded `item-invalid`, a new code, under a hint saying calendula sends the item unchanged; a Google event write answered 400 too. Line K25 of the CLI alignment round, twin of cardamum's C29 (`card-invalid`). Capabilities **backends** and **commands** moved.

- **The code**: `item-invalid`, by pimalaya-cli's noun rule (`item-` for calendars, `<noun>-invalid`); no code of calendula's or pimalaya-cli's table said it.
- **src/caldav/client.rs**: `rejects_item`, a 400 or a 403 naming `supported-calendar-data`, `valid-calendar-data`, `valid-calendar-object-resource`, `supported-calendar-component` (RFC 4791 §5.3.2.1) or the iCalendar, read by `status_code` after `need-privileges`, so a 403 whose plain body names the iCalendar no longer reads `permission-denied`. **src/caldav/backend.rs**: `invalid_item`, the hint on the same predicate, on `create_item` and `update_item`.
- **src/gcal/client.rs** `invalid_event`, on `events.insert`, `events.import` and `events.update` in src/gcal/backend.rs.
- **Not covered**: Graph, whose 400 on a write names no body recognisably (`ErrorInvalidRequest`, a quota and a malformed id come as 400 too).
- **Evidence**: `a_refused_write_carries_its_code` (a 403 `valid-calendar-data` read `None`), `a_refused_icalendar_is_invalid_and_keeps_the_answer_as_its_cause` and `an_event_write_answered_400_is_invalid` read `None`, or `permission-denied` for the plain 403, on the code before.

CHANGELOG (Added), MIGRATION.md (the code list). Spec: backends ("A server's refusal carries its code"), commands ("A JSON error carries a stable code").
