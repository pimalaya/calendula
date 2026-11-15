---
cairn: delta
change: the-agenda-stamps-local-times
---

# Delta

## ADDED Requirements

## MODIFIED Requirements

### Requirement: An agenda expands recurring series
(commands) Added: the agenda SHALL stamp each event, and file it under its day, at its start in the local zone `event list` reads (`TZ`, then the system's), a timed event at its instant there, a whole-day one on its day, a floating one at its wall time; the text and the keys of `starts` alike.

#### Scenario: A UTC event under a local zone
- GIVEN `TZ=Europe/Paris` and an event `DTSTART:20261112T090000Z`
- WHEN `calendula --json event agenda 11 2026` runs
- THEN the event is under `starts` at `2026-11-12T10:00:00`, as `event list` prints it at `2026-11-12 10:00`

## REMOVED Requirements
