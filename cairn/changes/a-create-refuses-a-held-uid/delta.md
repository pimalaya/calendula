---
cairn: delta
change: a-create-refuses-a-held-uid
---

# Delta

## ADDED Requirements

## MODIFIED Requirements

### Requirement: A server's refusal carries its code
(backends) The `item-duplicate` row SHALL read: a CalDAV create refused for `no-uid-conflict` (RFC 4791 §5.3.2) or answered 412 to its `If-None-Match: *`, its resource name being derived from the UID; a gcal create whose UID the calendar holds; a pimdir create naming a `UID` its calendar holds or has queued. Graph and vdir check no UID: a second create of one UID makes a second item there.

### Requirement: Google Calendar backend
(backends) Added: before it inserts or imports an event carrying a UID, `create_item` SHALL look the UID up (`events.list` filtered on the `iCalUID`, every page until one holds a match, a deleted event not listed) and SHALL refuse a UID the calendar holds with `item-duplicate`, ``Calendar `<cal>` already holds `<uid>` as event `<id>`; update that event instead``, `<id>` the series master's, sending no write: an import matches on the `iCalUID` and would overwrite that event.

#### Scenario: A UID the calendar holds
- GIVEN a gcal calendar holding an event of UID `e1@example.org`
- WHEN `calendula --json event add` of another event carrying that UID runs
- THEN it exits 1 coded `item-duplicate` naming the held event, and neither `events.insert` nor `events.import` is sent

### Requirement: A JSON error carries a stable code
(commands) The `item-duplicate` row SHALL read: a create names a `UID` its calendar holds: a CalDAV `no-uid-conflict` or 412, a gcal calendar holding it, a pimdir calendar holding or queuing it.

## REMOVED Requirements
