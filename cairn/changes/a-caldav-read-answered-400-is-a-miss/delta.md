---
cairn: delta
change: a-caldav-read-answered-400-is-a-miss
---

# Delta

## ADDED Requirements

## MODIFIED Requirements

### Requirement: A server's refusal carries its code
(backends) The `item-not-found` row SHALL read: an item request answered 404 or 410 (CalDAV, Google Calendar), a CalDAV item read answered 400 (iCloud's answer to a name it does not take, without `.ics`), a Graph 404 or 400 `ErrorInvalidIdMalformed`; a vdir item file or a pimdir item that does not exist. A CalDAV write answered 400 SHALL carry no code.

#### Scenario: An id iCloud does not take
- GIVEN a CalDAV server answering 400 to a GET of `<calendar>/nope`
- WHEN `calendula --json item read nope` or `event delete nope` runs
- THEN it exits 1 coded `item-not-found`, the server's answer in `sources`

## REMOVED Requirements
