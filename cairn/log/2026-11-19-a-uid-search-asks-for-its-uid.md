---
cairn: log
change: a-uid-search-asks-for-its-uid
landed: 2026-11-19
---

# A UID search asks for its UID

Landed without a proposal: the gcal performance half of CAL-9 (deep analysis of 2026-11-15).

`GcalBackend::list_items` sends the query's `uid` hint as the `iCalUID` filter of `events.list` on a listing taking every item, the one `event search <UID>` makes, as pimdir honours the hint; a paged listing ignores it, as commands.md says. Google lists the series master and its exceptions under one `iCalUID`, so the listing folds as before; `event search` walked every page of the calendar for one UID. The `uid` field of `CalendarItemQuery` is live under `gcal` too.

Test `a_uid_search_lists_through_the_ical_uid_filter` (src/gcal/backend.rs), which fails with the filter left out: one GET carrying `iCalUID=e1%40example.org`, and none on a paged listing.

Moved: commands.md (the `UID` hint of the listing query).

Left of CAL-9: the CalDAV and Graph halves, written as cairn/changes/a-page-reads-its-own-bodies, waiting on an owner ruling (may an unreadable row hold its place on a page).
