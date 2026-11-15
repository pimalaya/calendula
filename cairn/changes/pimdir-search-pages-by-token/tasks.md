---
cairn: tasks
change: pimdir-search-pages-by-token
---

# Tasks

- [x] Tests: src/pimdir/search.rs `a_hit_prints_on_one_line_and_an_error_last` and `a_page_names_the_next_and_its_errors`; src/pimdir/index.rs `a_hit_the_store_cannot_read_is_an_item_error` (an `event_summary` column of the wrong type, CHECKs ignored, through rusqlite, a new dev-dependency as in Himalaya), which failed on the code before, the whole search failing; tests/pimdir_search.rs over a filled and indexed store (`nextPage`, `--page-token`, the token winning, the text header, a refused token).
- [x] `PimdirClient::calendar_hit` and `PimdirCalendarPage::errors` (src/pimdir/index.rs); `PimdirCalendarCoverage::note` and `PimdirCalendarZones::note`, the count keeping its lines.
- [x] `PimdirSearchCommand` and `PimdirSearchOutput` (src/pimdir/search.rs).
- [x] CHANGELOG (Changed, breaking); MIGRATION.md (JSON output).
- [x] Fold the delta into [commands](../../spec/commands.md); log.
