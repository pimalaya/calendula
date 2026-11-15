---
cairn: change
change: zero-length-whole-day-ends-on-its-start
---

# Delta

## ADDED Requirements

(none)

## MODIFIED Requirements

### Requirement: Columns follow the kind
(Its second paragraph only.)

The event table SHALL show the start and end for a person to read, `startsAt` and `endsAt` in the local zone (`TZ`, then the system's), the one a floating time is read in: `2026-10-30 09:00` for a timed event, the days for a whole-day one with the end as its last day (`2026-05-19`), never before its first, so a zero-length one (`DTEND` equal to `DTSTART`) ends on its start. A time that did not resolve SHALL show as iCalendar spells it. Squeezed into the terminal or `--max-width`, the table SHALL truncate the id and the summary but keep the start and end whole. The `--json` payload SHALL stay as it is, `start` and `end` iCalendar-spelled and a whole day's `endsAt` exclusive.

## REMOVED Requirements

(none)
