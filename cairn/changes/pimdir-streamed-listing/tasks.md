---
cairn: tasks
id: pimdir-streamed-listing
---

# Tasks

- [x] pimdir `scan` visiting keyset batches until its visitor breaks, in place of `scan_items` collecting them
- [x] `list_items` keeping the page only, breaking once full, reading no body before an unfiltered page
- [x] `ruled_out`: kind and `UID` from the summary, never a date, never an item a pending `update` repoints
- [x] `CalendarItemQuery::uid`, set by `event find`, honoured by pimdir on an unpaged listing only
- [x] `pimdir status` counting as it walks
- [x] Tests: every kind, window and page against the whole-read listing on a store spanning several batches, with series, overrides, a masterless resource, an escaped `UID` and pending updates; bodies made unreadable outside the page, under another kind and under another `UID` are never read
- [x] Fold into [backends](../../spec/backends.md) and [commands](../../spec/commands.md); log entry; changelog
