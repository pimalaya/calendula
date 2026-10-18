---
cairn: change
id: gcal-reads-a-duration
status: landed
created: 2026-10-18
---

# A Google event may give its end as a DURATION

## Why

`event create --start 2026-10-30T09:00 --duration PT1H30M` on a Google account is refused: the gcal projection refuses a VEVENT whose end is a DURATION, since Google has no field for a length and requires an end. RFC 5545 3.8.2.5 allows either form, invitations often carry a DURATION, and `--duration` is a flag calendula offers on every backend (issue #14, the DURATION half).

## What

- io-gcal now computes the end from the DTSTART (pimalaya/io-gcal#1): the start shifted by the DURATION on its wall clock, keeping its shape (a `Z` time stays UTC, a TZID time keeps its zone, a date moves by whole days), and drops the DURATION, so the event reads back with a DTEND.
- calendula pins io-gcal at that revision (`9053761`) under `[patch.crates-io]` until it is released; no calendula code moves.
- A negative DURATION, an unreadable one, or one with hours after a `VALUE=DATE` start is still refused by name.
- The shift is on the wall clock, so `PT1H` across a daylight-saving change counts a nominal hour, as io-msgraph does.
