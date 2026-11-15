---
cairn: change
id: a-family-page-holds-that-family
status: landed
created: 2026-09-01
---

# A component listing paginated before it projected, so a todo could not be found

`todo create` reported the id it wrote and `todo list` showed nothing. The item was there: `item list` listed it, and `todo list -s 100` rendered it.

## Why

A family listing asked the backend for a page of items of *every* kind, then projected that page to its own kind. On a calendar of 30 events and one todo, the first 25 items are events, the projection keeps none, and the listing is empty. The page meant "the VTODOs among the first 25 items", not "the first 25 VTODOs".

The component kind never reached the backend at all, so nothing could have narrowed earlier. CalDAV's `calendar-query` takes exactly that filter and was only ever sent a hardcoded `VEVENT`, and then only when a date range was given.

## What

`list_items` takes a `CalendarItemQuery` carrying the page, the window and the component kind, and every backend narrows by kind before it paginates: CalDAV pushes it down as an RFC 4791 `comp-filter`, vdir and pimdir filter after parsing, gcal answers a non-VEVENT kind without a round-trip.
