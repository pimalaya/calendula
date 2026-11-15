---
cairn: log
change: the-d17-to-d21-rulings
landed: 2026-11-15
---

# The D17 to D21 rulings

Rulings D17 to D21 of the family's CLI alignment round land as pimalaya-cli's spec states them at 595504e. Capabilities **backends** and **commands** moved.

- **src/error.rs, src/vdir/backend.rs, src/vdir/create.rs, src/caldav/backend.rs, src/caldav/client.rs, src/msgraph/backend.rs**: `calendar-duplicate` on a create naming an existing calendar, on vdir (the directory exists), CalDAV (`DAV:resource-must-be-null`, or 405 or 412 with the calendar listed) and Graph (`ErrorFolderExists`); Google Calendar lets two calendars share a name, pimdir creates no calendar (D17).
- **src/msgraph/events/list.rs, src/msgraph/calendars/list.rs, src/msgraph/events/delta.rs and their outputs**: `nextPage`, Graph's `@odata.nextLink` lifted out of the page, and `--page-token`; the delta's `@odata.deltaLink` kept raw (D18).
- **src/pimdir/queue.rs, src/pimdir/queue/list.rs**: one `PimdirQueueRow`, `{row, kind, collection, linkId, itemId, producer, queuedAt, attempts, unconfirmed, error, summary}`, for the pending and the parked rows, what a parked row addresses read off its payload (D19, D21).
- **Evidence**: tests/calendar_duplicate.rs fails on the binary before (the second create exits 0); the unit tests caldav::backend `a_calendar_created_twice_is_a_duplicate`, msgraph::backend `a_calendar_named_twice_is_a_duplicate`, msgraph::client `a_graph_listing_pages_by_next_page` and `a_graph_delta_pages_by_next_page_and_keeps_its_delta_link`, pimdir::queue::list `the_rows_print_under_the_way_to_cancel_one` and pimdir::backend `a_parked_and_a_pending_row_list_in_one_shape` do not build on the code before, which has no `calendar-duplicate`, no `--page-token` on these commands and no shared row.

CHANGELOG (Changed, reworded Added lines), MIGRATION.md (D17). Spec: backends, commands.
