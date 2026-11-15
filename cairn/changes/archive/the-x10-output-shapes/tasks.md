---
cairn: tasks
change: the-x10-output-shapes
---

# Tasks

- [x] N0: .github/workflows/tests.yml gains `workflow_dispatch` and the weekly `0 4 * * 1` schedule, the `live` job guarded to run on them alone; CONTRIBUTING.md says so; actionlint passes. tests/google.rs and tests/microsoft.rs create no calendar: nothing to move onto a long-lived one.
- [x] A3: src/pimdir/intent.rs `PimdirIntentOutput` prints `row`, `id`, `calendarId`. Test failing before: pimdir::intent `a_queued_reply_prints_its_row_and_the_event_id`.
- [x] A11, A12: src/shared/event.rs `Event` prints `start` and `end` in RFC 3339 or date form, `uid` and `summary` nullable, the iCalendar spelling kept unprinted as `wire_start` and `wire_end` for windowing and the unresolved fallback; src/shared/event/expand.rs, list.rs, find.rs, src/shared/item.rs, src/pimdir/index.rs, search.rs, backend.rs follow. Tests failing before: shared::event::expand `the_json_shape_carries_people_and_the_meeting_link`, `absent_text_is_null_and_a_whole_day_prints_its_dates`, shared::event::build's JSON test.
- [x] A14: src/shared/event/agenda.rs prints `{events: [{id, start, label}], errors}`. Tests failing before: the agenda's JSON unit tests, tests/event_list.rs `the_agenda_json_holds_its_events_and_its_errors`, `the_agenda_stamps_an_event_in_the_local_zone`.
- [x] B13: src/gcal/free_busy.rs prints `queryErrors` raw. Test failing before: gcal::free_busy `the_api_errors_print_raw_under_their_own_key`.
- [x] CHANGELOG, MIGRATION.md; fold the delta; log.
