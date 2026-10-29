---
cairn: change
change: event-alarm-flag
---

# Delta

## MODIFIED Requirements

### Requirement: The composer is the authoring surface
The composer SHALL remain the complete authoring surface; the field flags cover the common fields only. A recurrence rule, and any alarm beyond one display alarm before the start, are where a flag stops being ergonomic, and the documentation SHALL say so, so their absence reads as the boundary it is.

### Requirement: The field flags are a convenience, not the surface
The field flags SHALL cover the fields `event list` renders and the few every calendar carries, and SHALL NOT grow to match what a composer models. Recurrence (`RRULE`, `RDATE`, `EXDATE`, overrides), every alarm but the one `--alarm` writes (several alarms, one after the end or related to it, an absolute one, an EMAIL or AUDIO one), conference links (`CONFERENCE`) and hand-written `VTIMEZONE`s are out; `--online-meeting` asks a sync engine to create a meeting and writes no link. The documentation SHALL say the composer is the complete surface.

### Requirement: The event projection
An event's JSON SHALL carry `id`, `etag` (the entity tag of its item, `null` when the backend has none), `uid`, `recurrenceId`, `summary`, `description`, `location`, `start` and `end` (iCalendar-spelled), `allDay`, `startsAt` and `endsAt` (RFC 3339 with offset, or `YYYY-MM-DD` with an exclusive end for a whole day), `timeZone`, `zoneAssumed` (`true` when `DTSTART` or `DTEND` names a `TZID` neither the time-zone database nor the calendar defines, read in the local zone), `recurring`, `sequence` (its `SEQUENCE`, 0 when absent), `status`, `transparency`, `busyStatus` (Outlook's), `organizer` (`email`, `name`), `attendees` (`email`, `name`, `partstat` defaulting to `NEEDS-ACTION`, `role`, `rsvp`, `cutype`), `onlineMeetingUrl` (`CONFERENCE`, then the vendors' properties) and `alarms` (every VALARM of the VEVENT in document order: `action` uppercased and `trigger`, its value as written, each `null` when absent). The end SHALL be `DTEND`, else the start plus `DURATION`, else a day for a date and the start for a time.

`event read` SHALL print the item's bytes, VALARMs included, and its JSON SHALL add the projected `events`, or the one occurrence `-r/--recurrence-id` names.

## ADDED Requirements

### Requirement: One display alarm can be set from a flag
`event build`, `event create` and `event update` SHALL take `--alarm <LEAD>`, `LEAD` being an RFC 5545 3.3.6 duration in any case, a leading `+` dropped, checked by ical-rs's validator, or a short form: a whole number followed by `m`, `h` or `d` (`15m` is `PT15M`, `1h` is `PT1H`, `2d` is `P2D`). A negative lead, or one that is neither, SHALL be refused naming both shapes.

`--alarm` SHALL drop every VALARM of the VEVENT and write one in place of the first, or after the VEVENT's last item when it carried none: `ACTION:DISPLAY`, `TRIGGER:-<LEAD>` and a `DESCRIPTION` (required for DISPLAY, RFC 5545 3.6.6) holding the summary the event ends up with, `Reminder` when it has none. Every other line SHALL keep its bytes.

#### Scenario: A create with an alarm
- GIVEN a CalDAV account
- WHEN `calendula event create --summary Stand-up --start 2026-10-19T09:00Z --end 2026-10-19T09:15Z --alarm 15m` runs
- THEN the event is written with `BEGIN:VALARM`, `ACTION:DISPLAY`, `TRIGGER:-PT15M`, `DESCRIPTION:Stand-up`, `END:VALARM` inside its VEVENT

#### Scenario: An update replacing two alarms
- GIVEN an event carrying a DISPLAY VALARM at `-PT10M` and an EMAIL VALARM at `-P1D`
- WHEN `calendula event update <ID> --alarm PT1H` runs
- THEN the event carries one VALARM, `TRIGGER:-PT1H`, where the first stood, and every other line keeps its bytes

### Requirement: Every alarm can be removed from a flag
`--no-alarm` SHALL drop every VALARM of the VEVENT and write nothing in its place, and SHALL conflict with `--alarm`. Removing from an event that carries no VALARM SHALL leave the event byte for byte. On a backend falling back to the calendar's default reminders when an event carries none (gcal), the event SHALL get those defaults, as the projection says.

### Requirement: A lead a backend cannot hold is refused before the write
`event create` and `event update` SHALL refuse an `--alarm` lead the account's backend cannot hold, by name and before reaching it, never rounding it: on gcal a lead that is not whole minutes or exceeds four weeks (40320 minutes); on msgraph a lead that is not whole minutes, or that names days or weeks, which the msgraph projection does not read as a reminder. `event build` reaches no backend and SHALL write any lead the grammar allows.

#### Scenario: A lead Google refuses
- GIVEN a gcal account
- WHEN `calendula event create --summary Trip --alarm 30d` (or `--alarm P5W`) runs
- THEN the command fails naming the four-week ceiling, and nothing is written
