---
cairn: tasks
change: a-read-prints-its-list-row
---

# Tasks

- [x] src/shared/item.rs: `CalendarItem` prints as `ItemRow` (`{id, calendarId, etag}` and the flattened `ItemContents`, `{contents, encoding}`), its schema `CalendarItem`; base64-codec added, as Himalaya and cardamum depend on it.
- [x] src/shared/item/read.rs and src/shared/event/read.rs print the row, flattened; `printable` takes the bytes.
- [x] src/shared/event.rs: `description` and `location` are `Option<String>`; src/shared/event/expand.rs and src/pimdir/index.rs follow, and the agenda label.
- [x] Tests failing on the code before: tests/read.rs `a_read_prints_the_row_its_list_prints` (no `id`), `a_body_that_is_not_utf8_is_base64_in_json` (an integer array, then lossy text), `an_absent_description_or_location_is_null` (`""`); src/shared/event/build.rs reads `location` `null`. tests/common.rs (the live suites' shape check) takes `null` for both.
- [x] CHANGELOG, MIGRATION; fold the delta into commands; log.
