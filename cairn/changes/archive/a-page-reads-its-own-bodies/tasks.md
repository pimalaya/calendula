---
cairn: tasks
change: a-page-reads-its-own-bodies
---

# Tasks

- [x] Owner ruling: an unreadable row holds its place on its page (2026-11-19)
- [x] `CalendarItemList` carries the places its window covered (`places`); `CalendarItemList::into_page` reads `more` from them, `CalendarClient::list_items` calling it
- [x] CalDAV: `enum_items` then `multiget_items` for a window, the one REPORT kept for a whole listing; tests `a_caldav_page_reads_its_own_bodies` and `an_unreadable_row_keeps_its_place_and_no_item_shifts`, both failing with the window path disabled
- [x] Graph: the window cut before `reread` on a listing without a range; tests `a_graph_page_reads_its_own_masters` and `an_unreadable_master_keeps_its_place_and_no_event_shifts`, both failing with the cut disabled
- [x] backends.md ("CalDAV backend", "Microsoft Graph backend", "Each backend names the items it could not read"), CHANGELOG, log
