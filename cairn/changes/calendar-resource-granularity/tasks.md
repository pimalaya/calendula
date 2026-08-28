---
cairn: tasks
change: calendar-resource-granularity
---

# Tasks

- [x] pimdir SPEC.md 13: the granularity rule, the revised summary shape, the per-component sort key, the four DTSTART shapes and the recurring-item caveat; SPEC.md 11: `sort_key` named the one column exempt from byte-identical encoding.
- [x] src/pimdir/meta.rs: summarise the master (the component carrying no RECURRENCE-ID), read the TZID and VALUE parameters off the raw line, emit the revised shape, and resolve the key through the document's own VTIMEZONE with the fold, gap and unresolvable-zone rules.
- [x] src/pimdir/backend.rs: the range filter reads `dtstart` then `due` off the summary.
- [x] src/gcal/project.rs: RECURRENCE-ID for an exception, `set_to_ical` folding a set into one resource under the master's UID, and the zone minting extracted so it runs over the folded document.
- [x] src/gcal/backend.rs: a listing groups on `recurringEventId`, an orphan exception still files on its own, and `get_item` folds through an iCalUID-filtered listing.
- [x] Tests: the master wins over an override listed first; a zoned start resolves to the instant its VTIMEZONE puts in force; an undefined zone reads as floating rather than unknown; a date-only start reads at midnight; a VTODO orders on DUE and stays orderable with neither date.
- [x] Tests: an exception names the instance it replaces; a series and its exception fold into one resource under one UID; a folded exception gets the zone it names defined; a listing files an exception with its master and an orphan on its own.
- [x] The CHANGELOG entries, folded into the unreleased gcal and pimdir backend bullets rather than filed as fixes, since neither backend has shipped.
- [x] Fold the delta into [cairn/spec/backends.md](../../spec/backends.md) and [cairn/spec/projection.md](../../spec/projection.md); write [cairn/log/2026-08-16-calendar-resource-granularity.md](../../log/2026-08-16-calendar-resource-granularity.md).
