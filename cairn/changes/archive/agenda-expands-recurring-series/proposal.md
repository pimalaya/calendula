---
cairn: change
id: agenda-expands-recurring-series
status: landed
created: 2026-10-16
---

# The agenda expands recurring series over the days it draws

## Why

`event agenda` lists every event of the calendar through `Event::project`, which places a series once, at its first start. A yearly birthday created in 2019 therefore marks 2019 and never the month on screen, while `event list --from/--to` over the same days lists its occurrence: the two views of one calendar disagree, and the agenda is the one that is wrong.

On a Google calendar every birthday and anniversary is such a series, so the agenda leaves out exactly the events a month view is opened to see.

## What

- The agenda settles the grid first, then fetches the events over the days it draws (first day of the first month to last day of the last, both inclusive) and expands them with `Event::occurrences`, as `event list` does with a window: each occurrence marks its own day and lists at its own time, overrides applied and excluded dates dropped.
- The window goes to `list_items` as a `CalendarTimeRange`, so CalDAV and gcal narrow it server-side and the local backends after parsing, instead of the agenda reading the whole calendar.
- A grid chrono cannot place (a year before 1) falls back to the unbounded listing, each series at its first start, as before.
- The grid, the text and the `--json` shapes are unchanged.
