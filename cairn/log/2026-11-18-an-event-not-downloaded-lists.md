---
cairn: log
change: an-event-not-downloaded-lists
landed: 2026-11-18
---

# An event not downloaded yet lists from its summary

Landed without a proposal: CAL-15 with TRI-2 of the deep analysis, against the spec's own "pimdir is an availability-aware cache".

`CalendarItem` gains `stand_in`, the event a listing shows for an item whose body is not local, never written back. The pimdir backend fills it from the item's `text/calendar` summary through `index::event_hit`, the projection `pimdir search` already used, and `Event::try_project` and `Event::try_occurrences` answer it for an item with no bytes. `event list`, `event search` and `event agenda` therefore show the event where they showed nothing while the page line counted it; a series shows once at its start. A body the store fails to read keeps its `item-unreadable` entry and gets no stand-in. Every other backend leaves the field `None`.

Test `a_bodiless_event_lists_from_its_summary` (src/pimdir/index.rs), with and without a window; it fails on the old projection, which answered no event for an empty body.

Capabilities moved: backends.
