---
cairn: change
id: event-build-projects
status: landed
created: 2026-10-06
---

# `event build` projects what it built

## Why

A client holding a loose iCalendar, such as an invitation attached to a mail, wants it read the way calendula reads a calendar's events: zones through the database or the invitation's own `VTIMEZONE`, the end from `DTEND` or `DURATION`, organizer and attendees. `event build` already reads such a source with no account and no backend, but answers its bytes only, so the client parses them a second time on its own (MOA did, with a hand-written parser).

An invitation also needs what the projection leaves out: the calendar's `METHOD` (`REQUEST`, `CANCEL`, `REPLY`), the event's `SEQUENCE`, and whether a zone was only assumed.

## What

- `event build` under `--json` answers `{contents, method, events}`: the bytes as before, the calendar's `METHOD` uppercased (`null` without one), and its events as `event read` prints them. Text output is unchanged.
- The event projection gains `sequence` (0 when absent) and `zoneAssumed`: `true` when `DTSTART` or `DTEND` names a `TZID` neither the database nor the calendar defines, so the time was read in the local zone.
