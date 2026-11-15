---
cairn: delta
change: a-refused-item-is-coded
---

# Delta

## ADDED Requirements

## MODIFIED Requirements

### Requirement: A server's refusal carries its code
(backends) The table SHALL add the row `item-invalid`: a CalDAV write answered 400 or 403 whose body names a precondition on the item's content (RFC 4791 §5.3.2.1: `supported-calendar-data`, `valid-calendar-data`, `valid-calendar-object-resource`, `supported-calendar-component`) or the iCalendar itself (`icalendar`, `vcalendar`, case ignored), the predicate calendula's hint reads, so the two never disagree; a Google event write (`events.insert`, `events.import`, `events.update`) answered 400, calendula building every parameter of that request. The `permission-denied` row SHALL read "a CalDAV 403 whose body names no precondition nor the iCalendar, or `need-privileges`".

The paragraph under the table SHALL read: A refusal none of these name SHALL carry no code, its reason kept in `sources`. A CalDAV write answered 400 naming nothing of the item SHALL carry none. Graph and vdir check no UID: a second create of one UID makes a second item there. A CalDAV 403 naming another precondition is an item the server would not take, no matter of rights, and SHALL NOT read `permission-denied`: `no-uid-conflict` is `item-duplicate`, a content precondition `item-invalid` under calendula's hint, the server's answer its cause, and any other (`max-resource-size`, `min-date-time`...) carries none. Graph's refusal of an event body carries none: its 400 names no body recognisably.

#### Scenario: An iCalendar the server refuses
- GIVEN a CalDAV server answering a `PUT` 403 with a `DAV:error` naming `C:valid-calendar-data`
- WHEN `calendula --json event add -k cal e.ics` runs
- THEN it exits 1 coded `item-invalid`, its error the hint saying calendula sends the item unchanged, the server's answer in `sources`

### Requirement: A JSON error carries a stable code
(commands) The code table SHALL add, after `item-unreadable`: `item-invalid` | the server refused the item written as no iCalendar it takes (CalDAV, Google Calendar, see "A server's refusal carries its code"); nothing was written, and the item is what to fix, not the account's rights.

## REMOVED Requirements
