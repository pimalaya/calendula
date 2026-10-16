---
cairn: change
id: gcal-anchors-floating-times
status: landed
created: 2026-10-16
---

# A backend without floating times anchors the time flags in the local zone

## Why

`event create --start 2026-10-19T09:00 --end 2026-10-19T10:00` on a Google account is refused: the time names no zone, so it is floating, and the gcal projection refuses a floating DTSTART by name (projection capability), since the API needs an offset or a named zone. Outlook needs one too. The user has to add `--time-zone` to every create, although calendula already knows the zone they mean: the local one, which it reads every floating time in, `event list` included (issue #14).

## What

- On the gcal and msgraph backends, `event create` and `event update` anchor a `--start` or `--end` naming no zone in the local zone (`TZ`, then the system's), exactly as `--time-zone` would: a `TZID` and its `VTIMEZONE`.
- `--time-zone` keeps the last word, and a date, a `Z` time or a zone-prefixed time is left as it is.
- The backends that store a floating time (vdir, CalDAV, pimdir, JMAP) keep it floating.
- A local zone without an IANA name anchors nothing, and the write is refused by name as before.
- No new setting: `--time-zone` already covers the explicit case. A floating time in a hand-written source or in the composer is not touched; the projection still refuses it.
