---
cairn: change
change: event-build-projects
---

# Delta

## ADDED Requirements

### Requirement: An event build answers its projection
`event build` under `--json` SHALL answer `{contents, method, events}`: the iCalendar it built, the calendar's `METHOD` uppercased (`null` when it carries none), and every VEVENT projected as `event read` projects them, the item id empty and the etag `null`. Its text output SHALL stay the iCalendar alone. The other families' `build` SHALL keep answering `{contents}`.

## MODIFIED Requirements

### Requirement: The event projection
An event's JSON SHALL carry `id`, `etag` (the entity tag of its item, `null` when the backend has none), `uid`, `recurrenceId`, `summary`, `description`, `location`, `start` and `end` (iCalendar-spelled), `allDay`, `startsAt` and `endsAt` (RFC 3339 with offset, or `YYYY-MM-DD` with an exclusive end for a whole day), `timeZone`, `zoneAssumed` (`true` when `DTSTART` or `DTEND` names a `TZID` neither the time-zone database nor the calendar defines, read in the local zone), `recurring`, `sequence` (its `SEQUENCE`, 0 when absent), `status`, `transparency`, `busyStatus` (Outlook's), `organizer` (`email`, `name`), `attendees` (`email`, `name`, `partstat` defaulting to `NEEDS-ACTION`, `role`, `rsvp`, `cutype`) and `onlineMeetingUrl` (`CONFERENCE`, then the vendors' properties). The end SHALL be `DTEND`, else the start plus `DURATION`, else a day for a date and the start for a time.
