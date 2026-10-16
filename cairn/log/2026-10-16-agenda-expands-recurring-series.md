---
cairn: log
change: agenda-expands-recurring-series
landed: 2026-10-16
---

# The agenda expands recurring series over the days it draws

`event agenda` in `src/shared/event/agenda.rs` now settles the grid before it reads the calendar: `first_month`, extracted from `monthly`, gives the first month drawn and `drawn_days` the first and last days, which go to `list_items` as a `CalendarTimeRange`. The events are projected with `Event::occurrences` over that window, so a series marks every day of the grid it recurs on instead of once at its first start; a grid chrono cannot place keeps the unbounded listing.

Tested on one month, three months spanning a new year and a whole year, and against a Google calendar whose yearly birthdays the agenda used to leave out.

Capabilities moved: **commands** (an agenda expands recurring series, added).
