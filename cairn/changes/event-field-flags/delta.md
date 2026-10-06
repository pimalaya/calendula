---
cairn: change
change: event-field-flags
---

# Delta

## ADDED Requirements

### Requirement: A source, the field flags and the composer stack
`event build`, `event create` and `event update` SHALL take an iCalendar source, the field flags and `-i/--interactive` together, applied in that order: the source (or the minted seed, or for an update the item the backend holds) is the event to start from, each flag sets the property it names on its VEVENT, and `-i` opens the result in the composer. A command given no source, no field flag and no `-i` has nothing to write and SHALL say so.

### Requirement: The field flags are a convenience, not the surface
The field flags SHALL cover the fields `event list` renders and the few every calendar carries, and SHALL NOT grow to match what a composer models. Recurrence (`RRULE`, `RDATE`, `EXDATE`, overrides), alarms, conferences and hand-written `VTIMEZONE`s are out. The documentation SHALL say the composer is the complete surface.

### Requirement: A field flag sets its property and touches nothing else
Writing a field flag SHALL drop every instance of the property it names on the VEVENT and write the flag's own, a repeated flag writing one instance per value. Every other line SHALL keep its bytes. A time flag SHALL take a date, a local time, a `Z` time, or a time prefixed by an IANA zone (`Europe/Paris:2026-10-19T09:00`); `--time-zone` names the zone of the times that name none. A zone a flag names SHALL arrive with its `VTIMEZONE`, minted once from the time-zone database. On an update, `--start` given without `--end` or `--duration` SHALL move the end with it, keeping the event's length.

### Requirement: Flags apply to one event
A source holding several VEVENTs SHALL be refused when a field flag is set, since a flag rewrites one event and the others would be dropped or left inconsistent. With no flag the source passes through as written.

## MODIFIED Requirements

### Requirement: The composer is the authoring surface
The composer SHALL remain the complete authoring surface; the field flags cover the common fields only. A recurrence rule and an alarm are where a flag stops being ergonomic, and the documentation SHALL say so, so their absence reads as the boundary it is.

## REMOVED Requirements

(none: "calendula SHALL NOT grow per-property flags" is replaced by the modified requirement above)
