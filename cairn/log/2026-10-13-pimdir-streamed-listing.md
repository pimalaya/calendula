---
cairn: log
change: pimdir-streamed-listing
landed: 2026-10-13
---

# A pimdir listing streams the store instead of loading it

`list_items` in `src/pimdir/backend.rs` walks the collection through `scan`, one keyset batch of rows at a time, keeps only the page, and stops once the page is full; `pimdir status` counts through the same walk. A body is read to answer a kind or a window or to be returned, never before an unfiltered page, never for an item whose summary sits in another component's table, and, under the `uid` hint `event find` now sets on `CalendarItemQuery`, never for an item whose summary states another `UID`. Dates rule nothing out, and an item a pending `update` repoints is always read. `paginate` is now built only for the backends that cut a whole list.

What a listing holds moves from every row and every body of the calendar to one batch of rows (500) and the page; a window, which lifts the page size, still reads every hydrated body, one at a time, and holds its matches.

Equivalence: a test lists every kind, five windows and eight pages of a store spanning several batches (series, an override moved into a window, a masterless resource, an escaped `UID`, to-dos, journals, a VFREEBUSY), before and after staged updates that change a kind and a `UID`, against the whole-read listing, and nine `UID` hints against it; another makes bodies unreadable and lists around them. The `--json` output of 158 commands (`event list` windows and pages, `event find`, `event read`, `item list`, `item read`, `todo list`, `journal list`, `pimdir status`) is byte-identical between `a8e873d` and this change on such a store with pending updates.

Capabilities moved: **backends** (a pimdir listing streams the store, added) and **commands** (a `UID` hint narrows a find, added).
