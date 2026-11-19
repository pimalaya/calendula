---
cairn: change
id: a-page-reads-its-own-bodies
status: landed
created: 2026-11-19
---

# A page reads its own bodies

## Why

CAL-9 of the deep analysis of 2026-11-15, its two performance halves left by a-graph-event-is-read-through-its-calendar (2026-11-18) once the gcal half landed (a-uid-search-asks-for-its-uid, 2026-11-19):

- CalDAV: a paged `item list` or `event list` without a range runs one `calendar-query` REPORT with `calendar-data`, so every body of the calendar travels for each page, where io-webdav 0.11 offers `enum_items` (ids and ETags) then `multiget_items` (the window's bodies).
- Graph: a paged listing without a range reads each series master again (one GET plus the instance windows) before it cuts the page, 150 to 250 sequential requests for one page of a calendar with 20 open series, again for page 2.

Both are cheap to write; what blocks them is a rule of the spec, not code. backends.md says a CalDAV listing names its unreadable rows "on the first page only: the whole calendar is read before the page is cut, and such a row holds no place on a page", and the Graph listing clears its errors past the first window for the same reason. A page that reads only its own bodies cannot know which rows before it are unreadable, so a row that fails holds no place only if every row before it was read: the pages would shift (an item shown twice or never) whenever a row fails, unless a failed row holds its place.

## What (ruled yes by the owner on 2026-11-19)

A paged listing without a range reads only its window's bodies, and a row it cannot read holds its place on its page, as pimdir already lists an unreadable body as an item with no bytes:

- `CalendarItemList` gains `places`, the ids of the listed places its window covers, and `CalendarItemList::into_page` tells `Next page:` from places, not from readable items;
- CalDAV: `enum_items`, cut the window over the enumerated references (their order is the order `list_items` returns, both sets sorted by id), `multiget_items` the window plus one, its unreadable rows named on that page; a whole listing (`take` none) keeps the one REPORT;
- Graph: window the unique listed events before `reread`, read again only those, an error naming its event on its page; a listing with a range keeps reading every master, its membership known only after the reread folds the cancelled occurrences;
- the spec sentence "on the first page only ... holds no place on a page" becomes "on the page whose places hold it", for CalDAV and Graph.

The cost: two round trips for a CalDAV page in place of one (the bodies of one page against those of the calendar); none on a whole listing.

Alternative: keep the rule and leave both halves as they are; CAL-9 stays a documented cost on large calendars.

## Owner ruling

An unreadable row holds its place on its page (owner, 2026-11-19).
