---
cairn: log
change: a-family-page-holds-that-family
landed: 2026-09-01
---

# A page of a family now holds that family

`todo create` reported the id it had written and `todo list` showed an empty table. The item was in the calendar: `item list` listed it, and `todo list -s 100` rendered it.

**The page was taken over the wrong set.** A family listing asked its backend for a page of items of every kind, then projected that page to its own kind. On a calendar of thirty events and one to-do, the first twenty-five items are events, the projection keeps none of them, and the listing is empty however many to-dos the calendar holds. The page meant "the VTODOs among the first 25 items", which is not what a page of `todo list` says it is. `event list` and `journal list` had it too, and only `item list`, which projects nothing, was right.

The component kind never reached a backend, so nothing could have narrowed earlier even if it had wanted to. CalDAV's `calendar-query` takes exactly that filter, and calendula sent it a hardcoded `VEVENT`, and then only when a date range was given.

**One query, narrowed in one order** (shared/item.rs `CalendarItemQuery`): the page, the window and the kind travel together, so no backend can serve one without the others, and every backend narrows by kind, then by window, then paginates. The five listings pass their family's kind through `IcalFamily::kind`, and `item` passes `None`, which is what makes it the unfiltered view.

**CalDAV pushes it down** (caldav/backend.rs `comp_filter`): the kind becomes the `comp-filter` and the `time-range` nests inside it, as RFC 4791 9.7.1 wants. A Posteo calendar of a thousand events now sends the to-dos and nothing else, where before it sent everything and calendula threw the events away. The hardcoded `VEVENT` is gone with it.

**gcal answers without asking** (gcal/backend.rs): Google models a VEVENT and nothing else, so `todo list` against it returns empty rather than paging through a listing that could never hold one.

**pimdir answers from the summary when the body is not there** (pimdir/backend.rs `is_kind`): the `v: 1` summary names the component a reader renders the resource as (pimdir SPEC Annex A.3), which is what keeps the listing availability-aware. Narrowing on the bytes alone would have made every not-yet-downloaded item vanish from `todo list`, which is the bug this change is fixing, one layer down.

**A runtime kind test** (shared/ical.rs `holds_kind`): the projections are keyed on a marker type, which a listing narrowing by a kind it only knows at runtime cannot use, so this reads the decoded component names. Bytes that do not parse carry no kind, which is what a projection makes of them too.

Verified: 104 tests green, one new over `holds_kind`; the reported case reproduced on a vdir store of thirty events and one to-do, empty before and correct after, with `event list` still paging events and `item list` still raw.

Capabilities moved: commands (one requirement added).
