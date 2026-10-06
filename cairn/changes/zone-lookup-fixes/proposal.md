---
cairn: change
id: zone-lookup-fixes
status: landed
created: 2026-10-06
---

# Two zone lookups that misread an invitation

## Why

Reading invitations for MOA through `event build` showed two misreadings. A `TZID` with a leading slash (`/Europe/Paris`) was never looked up whole: empty segments were dropped, then only the trailing ones tried, so the time fell to the local zone. A `VTIMEZONE` with no observance was taken as a defined zone, which ical-rs resolves to UTC, so `09:00` in Paris read as `09:00Z`.

## What

The segments of a `TZID` are tried from the first on. A `VTIMEZONE` stating no observance defines nothing: the time is read in the local zone and `zoneAssumed` says so.
