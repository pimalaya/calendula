---
cairn: change
id: event-delete-by-id-or-uid
status: landed
created: 2026-10-04
---

# `event delete` takes the id or the UID

## Why

`event delete` documented its argument as the iCalendar `UID`, yet passed it to the backend as the item id: on pimdir that is the store's number, so a `UID` failed as an invalid id. `item delete`, `todo delete` and `journal delete` carried the same wording.

## What

`event delete` takes either: the item id is tried first (a read), then the calendar's events are listed and the id looked for among their ids, then their `UID`s. A `UID` carried by several items is refused, naming their ids. The other delete commands document the id their listing reports.
