---
cairn: log
date: 2026-08-09
change: component-families
---

# One shared family per component kind

`todo` (VTODO) and `journal` (VJOURNAL) joined `event` (VEVENT), so every component kind a calendar stores now has a view of its own. Before, only events had one: a task list read through `item` showed an id, an ETag and a byte count, and telling a finished task from a pending one meant reading the raw bytes of every resource.

## What landed

Two projections and two command families, each mirroring `event`. `Todo` pulls out the summary, DUE, STATUS, PRIORITY and PERCENT-COMPLETE; `Journal` pulls out the summary, DTSTART and STATUS, a journal entry being a dated note with no end. Both drop every other component kind, and an item whose bytes do not parse projects nothing rather than failing the listing, exactly as `event` already did.

Nothing reached the backends. A component family lists items, projects its kind and renders a table, so the whole change is a projection and a table per kind plus the configuration blocks (`todo.list`, `journal.list`) that name their page size and column colours.

Three decisions worth recording:

- **The window is applied locally, not pushed down.** `event list` hands its range to the backend, which narrows server-side. A todo carries a due date and no start, and a journal entry a date and no end, while a server-side range filter is defined against a component's start and end (RFC 4791 9.9). Pushing the window down would drop them for the wrong reason, so the two new families filter after parsing. `CalendarTimeRange::contains` lost its `vdir`-or-`pimdir` cfg gate as a result: it is now needed whatever the backend.
- **A completed todo reads as done even with no percentage.** RFC 5545 3.8.1.8 makes `STATUS:COMPLETED` imply full completion, so the DONE column shows 100% for it rather than empty; an explicit PERCENT-COMPLETE still wins.
- **No agenda for the new kinds.** The grid marks the days that carry an event because an event occupies time. A due date and a note's date do not fill a day, and drawing them the same way would claim they do.

## Left out

VFREEBUSY and VTIMEZONE get no family: the first is the answer to a query rather than a stored resource (and where a backend offers that query, it belongs to that backend's own family, which is where gcal put it), and the second defines the zones the other components reference.

A `done` verb marking a task complete was considered and declined: completing a task is a partial update of STATUS and PERCENT-COMPLETE, and calendula edits iCalendar by being handed iCalendar. It would have been the first exception to that.

## Capabilities moved

- **commands**: `event against item` became `the component families against item`; added one family per stored kind, the columns-follow-the-kind rule, the agenda restriction, the locally-applied window and the kinds with no family.
- **config**: every listing family now carries its own rendering block.
