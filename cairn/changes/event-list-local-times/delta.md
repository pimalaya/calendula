---
cairn: change
change: event-list-local-times
---

# Delta

## ADDED Requirements

(none)

## MODIFIED Requirements

### Requirement: Columns follow the kind
A component listing SHALL render the properties its kind is read by: an event its summary and its start and end, a todo its summary, due date, status, priority and completion percentage, a journal its summary and its date.

The event table SHALL show the start and end for a person to read, `startsAt` and `endsAt` in the local zone (`TZ`, then the system's), the one a floating time is read in: `2026-10-30 09:00` for a timed event, the days for a whole-day one with the end as its last day (`2026-05-19`). A time that did not resolve SHALL show as iCalendar spells it. Squeezed into the terminal or `--max-width`, the table SHALL truncate the id and the summary but keep the start and end whole. The `--json` payload SHALL stay as it is, `start` and `end` iCalendar-spelled and a whole day's `endsAt` exclusive.

A property the component omits SHALL render empty rather than absent, so the columns line up down the table. A todo whose STATUS is `COMPLETED` SHALL read as fully done even where PERCENT-COMPLETE is absent, since RFC 5545 3.8.1.8 makes the one imply the other.

### Requirement: A window expands recurring events
(Its last paragraph only.)

Without a window the listing SHALL be the raw one, in the order the backend returns the items, a series once at its first start; it SHALL NOT be sorted by start, since only the first page the backend returns would be, which reads like an agenda without being one. The help SHALL point to `--from` (or `event agenda`) for what is coming.

## REMOVED Requirements

(none)
