---
cairn: log
change: a-graph-event-is-read-through-its-calendar
landed: 2026-11-18
---

# A Graph event is read through its calendar

Landed without a proposal: the correctness half of CAL-9.

`MsgraphBackend::check_held` reads `calendars/<calendar>/events/<id>?$select=id` (`me/` or `users/<id>/`, each segment percent-encoded) before `get_item`, `update_item` and `delete_item`, a 404 failing `item-not-found` with ``Item `<id>` not found in calendar `<calendar>` ``, another refusal keeping its own code and wording. io-msgraph's `event_get` addresses an event by id alone, so `item read -k A <id of B>` printed `"calendarId":"A"` and `event delete -k A <id of B>` deleted from B. It costs one request per read, update or delete. Test `an_event_of_another_calendar_is_not_found`; the scripted tests answering a read, `a_moved_change_key_refuses_the_delete_as_a_precondition` and `no_instances_request_spans_five_years_or_more`, answer the extra request first.

Left of CAL-9, performance only: a Graph listing still reads each series master in range again before cutting the page (2e967d1 of 2026-11-16 already leaves out a series that cannot reach the range); CalDAV downloads every body of a calendar for each page where io-webdav offers an enumerate then multiget pair; a Google `event search <uid>` walks every page where `iCalUID` filters. An io-msgraph `event_get` scoped to a calendar would save the extra request.

Capabilities moved: backends.
