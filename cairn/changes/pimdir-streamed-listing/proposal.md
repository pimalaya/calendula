---
cairn: change
id: pimdir-streamed-listing
status: landed
created: 2026-10-13
---

# A pimdir listing streams the store instead of loading it

## Why

A pimdir listing gathered every row of the calendar into memory, then read every body before its kind and window filters ran, then cut the page out of the whole list: a page of ten held every body of the calendar at once. `event find` did the same to compare one `UID`, and `pimdir status` held every row to count them. On a large calendar that is memory growing with the store rather than with the answer, the failure the Pimalaya Android client met, and work a page or a find never needed.

## What

- `list_items` walks the collection in keyset batches, holding one batch of rows, one body at a time and the page it returns, and stops reading once the page is full.
- Before a page with no filter, an item is counted off its row: no body is read.
- An item the stored summary proves of another kind is not read: a resource holds one component type, filed in that type's table (pimdir STORAGE Annex A.3).
- `CalendarItemQuery` carries an optional `uid`, a hint `event find` sets: pimdir does not read an item whose summary states another `UID` (RFC 4791 4.1, one `UID` per resource), the summary's verbatim value compared only when it holds no escape. The hint is ignored on a paged listing, and the other backends ignore it; `event find` still compares every event it gets.
- A summary never rules an item out by date: an override moves an occurrence anywhere and the summary describes the master alone, so a window is still answered from the body.
- An item a pending `update` repoints carries a summary of its old body, so it is never ruled out by its summary.
- `pimdir status` counts while it walks.

The output of every command is unchanged.

## Not in scope

Reading fewer bodies under a date window needs a summary that bounds a resource's whole span, its overrides included, or the occurrence index pimdir SEARCH §7 describes; neither exists in io-pimdir 0.8.
