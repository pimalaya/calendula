---
cairn: change
id: calendar-resource-granularity
status: landed
created: 2026-08-16
---

# The calendar item is the resource, not the component

## Why

[pimdir issue 1](https://github.com/pimalaya/pimdir/issues/1) reads the store's `PRIMARY KEY (collection, link_id)` against the way iCalendar identifies a recurring series and concludes that a series plus three modified instances is four rows contending for one key, since RFC 5545 3.8.4.4 gives an override the same UID as its master.

It proposes making the link id the UID plus the RECURRENCE-ID.

The diagnosis names a real hole and the remedy is the wrong one.

RFC 4791 4.1 requires the components sharing a UID to live in **one** calendar object resource, and that UID to be unique within the collection holding it, so at resource granularity `(collection, link_id)` is exactly the uniqueness CalDAV itself enforces.

A composite key would put four rows behind one href and one ETag, leave a push to reassemble the resource anyway (an override cannot be `PUT` alone), and fan a delete and a revive out over four keys.

What was missing is that nothing **stated** the granularity, which is what made the other reading available. pimdir SPEC 13 now does, and calendula is the connector that has to hold to it. Two of its own defects fall out of the same rule:

- The pimdir summary read `cst.components::<VEVENT>().next()`, the first VEVENT of the resource. A resource carrying overrides can list one first, so the item was summarised as the instance that moved rather than as the series.
- The gcal backend is where the divergence the issue predicted exists. Google is instance-granular: `events.list` returns a modified instance as an event of its own and the projection emitted no RECURRENCE-ID, so an exception filed alone and gcal disagreed with CalDAV on how many items a calendar holds.

A third defect surfaced while working through the issue's DTSTART analysis, and is the one that loses information rather than merely misplacing it.

`parse_stamp` reads a stamp on the wall clock, ignoring the TZID the lens hands over separately, and the summary then formatted it with a trailing `Z`. For every zoned event, `meta.start` asserted a UTC instant while carrying local time: on the issue reporter's calendar, 118 of 128 items.

## What

### The summary carries times verbatim

The `text/calendar` summary follows the revised SPEC 13: `component` rather than `kind`, DTSTART carried verbatim beside `dtstart_tzid` and `dtstart_value`, DTEND and DUE the same way, and `recurring` so a reader knows it must expand.

A reader with a time zone database re-derives an instant in its own zone, which is what a summary is for; a reader without one displays the wall time the calendar wrote instead of a UTC claim nobody can honour.

The resolved instant is not duplicated in the summary. The `sort_key` is already returned by the store's paging reads and is the single resolved projection, under one stated policy, so there are not two answers to the same question.

### The sort key resolves properly, and says what it does when it cannot

DTSTART for a VEVENT or a VJOURNAL, DUE then DTSTART for a VTODO, which is scheduled by its due date and need not carry a start at all.

A zoned value resolves through the VTIMEZONE the document carries, taking the earlier offset of a fold and the offset after a gap, since a local time at a transition names two instants or none.

Where the zone will not resolve, the wall time is read as UTC rather than the key being dropped.

The issue proposed the empty key there; on its own reporter's corpus that is 118 items sent to the far end of every listing, whereas a wall-clock reading is wrong by an offset and stays next to where it belongs. The empty key is kept for a value that does not parse at all.

### A recurrence set is one item, on every backend

The gcal projection emits RECURRENCE-ID for an exception, and a listing folds the exceptions of a series into the master's document under the master's UID.

Reading one series costs one extra listing filtered by its iCalUID, since the API offers no query for the children of an event and expanding the instances would return the occurrences the RRULE already generates rather than the modifications alone.
