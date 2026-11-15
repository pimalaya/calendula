---
cairn: change
change: gcal-reads-a-duration
---

# Delta

## ADDED Requirements

(none)

## MODIFIED Requirements

### Requirement: Boundaries carry a zone or nothing
An all-day boundary SHALL project as a `VALUE=DATE` property and back; a timed one anchored in a named zone SHALL carry a TZID parameter and Google's `timeZone`; a timed one in UTC SHALL carry the `Z` suffix.

A floating DTSTART (no `Z` suffix and no TZID) has no Google form, since the API needs either a UTC offset or a named zone, so it SHALL be refused by name rather than resolved to a guessed zone.

A VEVENT expressing its length as a DURATION rather than a DTEND SHALL be given the end that DURATION names, since Google requires an end and has no field for a length: the DTSTART shifted by it on its wall clock, in its zone and its shape (a date by whole days only), the DURATION dropped so the event reads back with a DTEND. A negative DURATION, an unreadable one, or one with hours after a `VALUE=DATE` start SHALL still be refused by name.

A recurring event whose boundaries are UTC SHALL be given the `UTC` time zone explicitly, since Google expands a recurrence in the zone of its start and a UTC offset names none.

That zone SHALL project back as the `Z` suffix rather than as a TZID, so the projection is idempotent: projecting a document a second time reproduces it byte for byte.

#### Scenario: A create with a duration on a Google account
- GIVEN a gcal account
- WHEN `calendula event create --time-zone Europe/Madrid --start 2026-10-30T09:00 --duration PT1H30M` runs
- THEN Google stores an event from 09:00 to 10:30 Europe/Madrid, and it reads back with `DTEND;TZID=Europe/Madrid:20261030T103000` and no DURATION

## REMOVED Requirements

(none)
