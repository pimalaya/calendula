---
cairn: delta
change: the-x10-output-shapes
---

# Delta

## ADDED Requirements

## MODIFIED Requirements

### Requirement: The event projection
(commands) `uid`, `summary`, `description` and `location` SHALL be `null` when the event carries none, never `""`. `start` and `end` SHALL be RFC 3339 with the offset, a zero one spelled `Z`, or `YYYY-MM-DD` with an exclusive end for a whole day, `null` when the time does not resolve; `startsAt` and `endsAt` are gone, and a time's iCalendar spelling stays in the item's contents.

### Requirement: Columns follow the kind
(commands) The text shows the JSON `start` and `end` in the local zone; the `--json` payload is the event projection, a whole day's `end` exclusive.

### Requirement: An agenda shows every event at an instant
(commands) The `--json` payload SHALL be `{events, errors}`, `events` listing every event under the grid as `{id, start, label}` in the order the text renders them, `id` the item id `event read` takes, `start` the event projection's.

### Requirement: An agenda expands recurring series
(commands) The local zone stamps the text alone; the JSON `start` stays the event's instant as the event projection prints it. Scenario *A UTC event under a local zone*: the grid lists it at `Nov 12, 10:00` and its JSON `start` is `2026-11-12T09:00:00Z`.

### Requirement: gcal family
(commands) `free-busy` SHALL print `{calendars: [{id, busy: [{start, end}], queryErrors}]}`, `queryErrors` the API's error objects (`domain`, `reason`) raw, never under `errors`.

### Requirement: pimdir queues scheduling intents
(backends) `pimdir reply` and `pimdir cancel` SHALL print `{row, id, calendarId, recurrenceId, partstat, comment, source}`, `row` the queue row and `id` the event.

### Requirement: Live provider suites
(packaging) CI SHALL run the live suites by hand and weekly (Mondays, 04:00 UTC), never on push, and a release by hand on the commit to tag. A test needing a calendar SHALL find a long-lived one by the fixed summary `pimalaya-live-calendula-<purpose>`, create it only when absent, never delete it, and clear the events it owns at its start and end.

## REMOVED Requirements
