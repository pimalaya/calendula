---
cairn: change
id: the-live-night-findings
status: active
created: 2026-11-24
---

# The live night findings

The live test night of 2026-11-23 (workspace .lanes/live/report.md) ran every calendula command on CalDAV (Stalwart, Fastmail, iCloud, posteo read), JMAP (Stalwart, Fastmail), Google, Graph, vdir and every pimdir account. 2496f83 fixed iCloud refusing `If-Match: *`. The online meeting gaps and the delete reported `applied` are neverest's (its cairn/changes/the-live-night-findings).

## What

1. **Fastmail JMAP refuses every event write**: "CalendarEvent create rejected: invalidProperties (`iCalendar`)". The `iCalendar` member is ical-rs's round-trip escape hatch (`jscalendar/export.rs:542`), which projection.md relies on; Stalwart takes it. Recommendation: on `invalidProperties` naming it, send the write again once without it and warn naming what the server will not keep. Ask Fastmail whether the member is accepted under another capability.

## Not tested, for the next live run

microsoft-pimdir calendars on the shared store (the live neverest rc syncs Graph mail only), draining `pimdir reply` and `cancel` on a real account (with a guest of the run's own, never example.org), `-i` and `--composer`, the google and microsoft live suites.
