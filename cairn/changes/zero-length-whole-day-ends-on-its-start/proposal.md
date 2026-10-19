---
cairn: change
id: zero-length-whole-day-ends-on-its-start
status: landed
created: 2026-10-19
---

# A zero-length whole day ends on its start

## Why

`event-list-local-times` shows a whole-day event's end as the day before its exclusive `endsAt`. A whole-day event whose `DTEND` equals its `DTSTART` (zero length, which some writers produce) gets an `endsAt` equal to its `startsAt` (`Span::of` clamps the days at zero), so the table showed it ending the day before it starts: `2026-05-19` to `2026-05-18`.

## What

- The table's last day is never before the first: a zero-length whole-day event reads `2026-05-19` to `2026-05-19`, as a one-day event does.
- The `--json` payload is unchanged: `endsAt` still equals `startsAt` there.
