---
cairn: change
id: the-agenda-stamps-local-times
status: landed
created: 2026-11-15
---

# The agenda stamps local times

> Line K23 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), from the live testing campaign of 2026-11-15 (its reports/calendula/contract.md C-F1).

## Why

An event `DTSTART:20261120T090000Z`: `event list` printed it at 10:00 under the system zone CET and at 04:00 under `TZ=America/New_York`, and `event agenda` at 09:00 under both, its JSON key `2026-11-20T09:00:00`. `Event::start_at` parsed the stamp as written, a `Z` dropped, so the agenda stamped a UTC event at its UTC wall time. Every Google event, read with a `Z`, met it. commands.md puts starts in the local zone ("Columns follow the kind") and wants the agenda to agree with a windowed listing of the same days.

## What

- `Event::start_at(zone)`: a timed event at its instant on `zone`'s wall clock, a whole day at midnight on its day, a start that did not resolve as written.
- `event agenda` stamps and files each event in the local zone `event list` reads (`TZ`, then the system's), its text and its JSON keys alike.
- Spec: commands ("An agenda expands recurring series").
