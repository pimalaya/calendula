---
cairn: log
change: a-page-reads-its-own-bodies
landed: 2026-11-19
---

# A page reads its own bodies

The CalDAV and Graph performance halves of CAL-9 (deep analysis of 2026-11-15), on the owner's ruling of 2026-11-19: an unreadable row holds its place on its page, as pimdir lists an unreadable body as an item with no bytes (cairn/changes/archive/a-page-reads-its-own-bodies).

- **Shared paging**: `CalendarItemList` gains `places`, the ids of the listed places a window covers, set by a backend cutting its window before it reads the bodies. `CalendarItemList::into_page` cuts the window, read one place past the page, into the page: another page follows when a place lies past it, and that place's item and entry are dropped. `None` keeps the old count of items read.
- **CalDAV**: a paged listing (`item list`, `event list` without a range) runs a `calendar-query` asking `getetag` alone (io-webdav `enum_items`), cuts its places over the references, and fetches their bodies alone through one `calendar-multiget` (`multiget_items`), its rows put back in place order. A refused row is an entry on its own page; a place the multiget answered nothing for is an `item-unreadable` entry, "The server sent nothing for the item". A truncated enumeration is named on the first page. A listing taking every item keeps the one REPORT with `calendar-data`. The cost: two round trips per page in place of one, carrying one page's bodies (plus the one past it) where every body of the calendar travelled.
- **Graph**: a paged listing without a range windows the unique listed events before `reread`, so it reads again only the series masters of its page (and the one past it); a master refused on its reread is an entry on its own page, one gone (404) holds its place with no entry. A listing with a range keeps reading every master in range and naming its errors on the first page, a master's place known only once its cancelled occurrences are read.

Tests, each seen failing with the new path disabled (the old code path) before it passed: `a_caldav_page_reads_its_own_bodies` (no `calendar-data` asked by the query, the multiget naming `e2.ics` and `e3.ics` alone), `an_unreadable_row_keeps_its_place_and_no_item_shifts` (pages of 2 over four items, `e2.ics` refused: page 1 shows `e1.ics` and the entry, page 2 `e3.ics` and `e4.ics`), `a_graph_page_reads_its_own_masters` (one request, no master past the page read), `an_unreadable_master_keeps_its_place_and_no_event_shifts`. `a_caldav_listing_names_its_rows_on_the_first_window_only` is gone with the rule it held.

Moved: backends.md (CalDAV backend, Microsoft Graph backend, Each backend names the items it could not read).

Left: the live cells, a paged `item list` and `event list` on a real CalDAV server (multiget support, id spelling of the multiget rows against the enumeration's) and on a Microsoft 365 calendar.
