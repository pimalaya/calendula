---
cairn: tasks
change: the-d17-to-d21-rulings
---

# Tasks

- [x] D17: src/error.rs `CalendarDuplicate` and `calendar_duplicate`; src/vdir/backend.rs and src/vdir/create.rs refuse an existing directory; src/caldav/backend.rs `mkcalendar_refusal`; src/msgraph/backend.rs codes `ErrorFolderExists`. Tests failing on the code before: tests/calendar_duplicate.rs, caldav::backend `a_calendar_created_twice_is_a_duplicate`, msgraph::backend `a_calendar_named_twice_is_a_duplicate`.
- [x] D18: src/msgraph/events/list.rs, src/msgraph/calendars/list.rs, src/msgraph/events/delta.rs take `--page-token`; their outputs print `nextPage`. Tests failing before: msgraph::client `a_graph_listing_pages_by_next_page`, `a_graph_delta_pages_by_next_page_and_keeps_its_delta_link`.
- [x] D19, D21: src/pimdir/queue/list.rs `PimdirQueueRow` for both lists; src/pimdir/queue.rs reads what a row addresses off any action, a parked payload included. Tests failing before: pimdir::queue::list `the_rows_print_under_the_way_to_cancel_one`, pimdir::backend `a_parked_and_a_pending_row_list_in_one_shape`.
- [x] CHANGELOG, MIGRATION.md (D17); fold the delta; log.
