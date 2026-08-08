---
cairn: tasks
change: drop-io-calendar
---

# Tasks

- [x] Shared types owned in-repo: `Calendar`, `CalendarDiff`, `CalendarItem`, `CalendarTimeRange`.
- [x] `shared/client.rs` rewritten as an enum dispatcher over the compiled backends.
- [x] `src/caldav/backend.rs` and `src/vdir/backend.rs` adapting io-webdav and io-vdir.
- [x] ical-rs adopted; `Event::project` replaces `CalendarItem::as_ical` in the listing and the agenda.
- [x] Every dependency bumped; pimconf replaced by io-pim-discovery 0.5.
- [x] comfy-table v8: `shared/table.rs` maps the v7 preset string onto `TableStyle`.
- [x] Every backend feature combination builds; fmt and clippy clean; 43 tests pass.
