---
cairn: delta
change: an-event-is-named-by-its-id
---

# Delta

## ADDED Requirements

### Requirement: An event is named by its id alone
(commands) `event read`, `event update` and `event delete` SHALL take the id `event list` reports and never an iCalendar `UID`, which `event search` turns into that id: resolving a `UID` costs a listing of the whole calendar on every backend but pimdir. `event delete` SHALL read the id first: a read answering `item-not-found` SHALL fail the command coded `item-not-found`, its message naming `event search <UID>`, and list nothing; any other failure of the read SHALL fail the command with its code; a read answering `body-pending` (pimdir) names the item all the same. The item SHALL be deleted whole.

#### Scenario: A UID given to event delete
- GIVEN the vdir calendar `personal` holding `a.ics`, whose `UID` is `a@example.org`
- WHEN `calendula --json event delete a@example.org` runs
- THEN it fails coded `item-not-found`, naming `event search <UID>`, and `a.ics` stays

## MODIFIED Requirements

## REMOVED Requirements

### Requirement: An event is deleted by its id or its UID
(commands) Replaced by "An event is named by its id alone".
