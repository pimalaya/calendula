---
cairn: delta
change: a-read-prints-its-list-row
---

# Delta

## ADDED Requirements

### Requirement: An item's bytes are text or base64
(commands) An item SHALL print as one row in `item list`, `item read` and `event read`: `{id, calendarId, etag, contents, encoding}`, `etag` `null` when the backend versions no item. `contents` SHALL be the iCalendar text when its bytes are valid UTF-8, `encoding` then `utf-8`, and the base64 of the bytes otherwise (RFC 4648 standard alphabet, padded), `encoding` then `base64`; both SHALL be `null` when the item carries no bytes. No output SHALL decode an item's bytes lossily under `--json`.

#### Scenario: A body that is not UTF-8
- GIVEN the vdir calendar `personal` holding `a.ics`, whose `SUMMARY` holds the Latin-1 byte 0xE9
- WHEN `calendula --json item list`, `item read a` or `event read a` runs
- THEN each prints the file's bytes in base64 as `contents` and `"encoding":"base64"`, and `calendula item read a` prints `SUMMARY:Caf` and U+FFFD

## MODIFIED Requirements

### Requirement: The event projection
(commands) `description` and `location` SHALL be `null` when the event carries none. `event read` SHALL print the item's bytes, and its JSON SHALL be the row of its item ("An item's bytes are text or base64") plus the projected `events`, or the one occurrence `-r/--recurrence-id` names.

### Requirement: A delete can be gated on a version
(commands) `item read` SHALL print the row of its item ("An item's bytes are text or base64"), its `etag` the version read, `null` when the backend has none.

### Requirement: Text output is plain text
(commands) The JSON `contents` of `event read` and `item read` SHALL carry the stored bytes unchanged, as text or as base64; their text view SHALL read bytes that are not valid UTF-8 as U+FFFD.

## REMOVED Requirements
