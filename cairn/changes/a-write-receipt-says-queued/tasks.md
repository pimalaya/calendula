---
cairn: tasks
change: a-write-receipt-says-queued
---

# Tasks

- [x] `queued` serialized on `EventCreatedOutput`, `EventUpdatedOutput`, `ItemCreatedOutput`, `ItemUpdatedOutput`; their help names it.
- [x] `CalendarCreateOutput` in src/shared/calendar/create.rs, registered under `calendula-calendar-create`; `Write::Create` goes, no receipt using it.
- [x] Test failing on the code before: tests/write.rs `every_write_receipt_says_whether_it_is_queued` (no `queued` key, no `name`).
- [x] CHANGELOG, MIGRATION; fold the delta into commands; log.
