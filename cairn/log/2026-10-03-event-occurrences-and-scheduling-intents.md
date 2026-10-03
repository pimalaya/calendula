---
cairn: log
change: event-occurrences-and-scheduling-intents
landed: 2026-10-03
---

# Event occurrences, and replies and cancellations through pimdir

`event list --from/--to` now expands recurring events into the occurrences overlapping the window (src/shared/event/expand.rs, over ical-rs's `IcalRecurSet`), each carrying the item id and its `recurrenceId`, its times resolved to instants with their offset through the time-zone database, the calendar's `VTIMEZONE` or the local zone. The event JSON grew the people, the place, the status and the meeting link a calendar view shows. `event read -r` projects one occurrence, and `event find` looks an event up by `UID`. The local range filters keep an item by its occurrences, so a series begun before a window reaches it on vdir, msgraph and pimdir alike.

`pimdir reply` and `pimdir cancel` queue the `calendar-reply` and `calendar-cancel` intents of pimdir STORAGE Annex B.2 through the producer, naming their performer as §15.6 says.

jiff and the ical-rs `tzdb` feature are now always on, the projection being shared by every backend.

Capabilities moved: **commands** (a window expands recurring events, times resolve to instants, the event projection, an event is found by its UID; time-range filtering and the protocol-specific families, modified) and **backends** (pimdir queues scheduling intents).
