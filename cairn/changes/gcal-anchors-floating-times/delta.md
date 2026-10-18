---
cairn: change
change: gcal-anchors-floating-times
---

# Delta

## ADDED Requirements

### Requirement: A backend without floating times anchors the time flags locally
On a backend that cannot store a floating time (gcal, msgraph), `event create` and `event update` SHALL read a `--start` or `--end` naming no zone in the local zone (`TZ`, then the system's), as `--time-zone` naming that zone would, `TZID` and `VTIMEZONE` included. `--time-zone` SHALL win when given; a date, a `Z` time and a zone-prefixed time SHALL be left as they are. The other backends SHALL keep such a time floating. A local zone with no IANA name SHALL anchor nothing.

The local zone is the one calendula reads a floating time in, so the event lands at the wall-clock time `event list` will show it at.

#### Scenario: A create on a Google account
- GIVEN a gcal account and a system zone of `Europe/Madrid`
- WHEN `calendula event create --start 2026-10-19T09:00 --end 2026-10-19T10:00` runs
- THEN the event is written with `DTSTART;TZID=Europe/Madrid:20261019T090000` and its `VTIMEZONE`, and Google accepts it
