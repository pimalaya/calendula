---
cairn: tasks
change: the-x13-low-fixes
---

# Tasks

- [x] X9c: src/error.rs `calendar_duplicate(name)` and `existing_calendar(name, cause)`; src/vdir/backend.rs, src/vdir/create.rs, src/caldav/backend.rs, src/msgraph/backend.rs. Tests failing before: caldav::backend `a_calendar_created_twice_is_a_duplicate`, msgraph::backend `a_calendar_named_twice_is_a_duplicate`, tests/calendar_duplicate.rs.
- [x] K28: src/vdir/backend.rs `update_item` locates first; src/error.rs `read_failed` in the CalDAV, gcal, Graph and vdir `get_item`. Tests failing before: vdir::backend `an_update_of_a_missing_item_creates_nothing`, tests/read.rs `an_unknown_id_is_coded_not_found` (the two updates, one source).
- [x] Raw gcal listings: src/gcal/client.rs `raw_items`; src/gcal/calendars.rs, acl/list.rs, events/instances.rs, settings.rs print `Vec<Value>`; src/gcal/render.rs `str_of`, `boundary_of`. Tests failing before: each command's `..._as_google_sent_it` / `..._them`.
- [x] `attempts`: src/pimdir/queue/list.rs `u64`, src/pimdir/queue.rs. Test failing before: pimdir::queue::list `the_attempts_are_an_unsigned_count`.
- [x] D26: src/gcal/request.rs, wired in src/gcal/cli.rs and the schema registry. Tests: gcal::request.
- [x] D27: nothing calendula builds prints `Next page:` before its listing.
- [x] D28: src/shared/calendar/create.rs, the vdir hint; tests moved to `--id`; tests/write.rs refuses the old form.
- [x] CHANGELOG, MIGRATION.md; fold the delta; log.
