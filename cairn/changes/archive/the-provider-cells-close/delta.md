---
cairn: delta
change: the-provider-cells-close
---

# Delta

## ADDED Requirements

### Requirement: An update reads a zone-less time in the event's zone
(commands) On `event update`, a `--start` or `--end` naming no zone SHALL be read in the zone the event's `DTSTART` names in its `TZID`, when the time-zone database knows it, so the event moves on its own clock rather than turning floating; `--time-zone` SHALL keep the last word, and an event whose start names no such zone SHALL keep the reading of "A backend without floating times anchors the time flags locally".

#### Scenario: A zoned event moved by a local time
- GIVEN an event `DTSTART;TZID=Europe/Paris:20261112T090000` ending at 10:00
- WHEN `calendula event update <id> --start 2026-11-26T11:00` runs
- THEN it is written `DTSTART;TZID=Europe/Paris:20261126T110000` and `DTEND;TZID=Europe/Paris:20261126T120000`

## MODIFIED Requirements

### Requirement: A field flag sets its property and touches nothing else
(commands) The sentence on `--time-zone` SHALL read: `--time-zone` names the zone of the times that name none, and on `event update` such a time is otherwise read in the event's own zone ("An update reads a zone-less time in the event's zone").

### Requirement: gcal family
(commands) The ACL paragraph SHALL read: an ACL rule id is `<scope type>:<scope value>` (`default` for everyone), which the API mints, so `create` SHALL take the scope and the role as flags and derive it, `update` SHALL read the scope from the id it is given, its `--scope` and `--value` optional and refused `usage-invalid` before any request when they disagree with the id (the value compared case-insensitively), as an id naming no scope is, and `delete` SHALL accept the id a listing showed.

### Requirement: A JSON error carries a stable code
(commands) Added row, after `occurrence-unsupported`: `invitation-missing`, `pimdir reply` names an event naming no `ORGANIZER` to answer or no `ATTENDEE` to answer as, or `pimdir cancel` one naming no `ATTENDEE` to notify; nothing was queued. The `item-not-found` row SHALL name a Google event read cancelled, a deleted one.

### Requirement: CalDAV backend
(backends) The create sentence SHALL read: a create SHALL propose a resource name derived from the UID of the item's first VEVENT, VTODO or VJOURNAL when that UID is URL-safe, falling back to a content digest, and SHALL keep the id the server reports in its `Location` header when it names the resource itself, else the last segment of the URL io-webdav sent, percent-encoded, which is the form a listing returns.

### Requirement: Google Calendar backend
(backends) Added: a read of an event Google holds as cancelled, a deleted event's stub, SHALL be `item-not-found` (`event read`, `item read`, the base an update reads), as a 404 is; and the user's primary calendar (`primary` on its calendar list entry) SHALL be the `default` of `calendar list`.

### Requirement: Microsoft Graph backend
(backends) Added: an empty `hexColor`, Graph's spelling of no colour, SHALL read as none, and the calendar Graph marks `isDefaultCalendar` SHALL be the `default` of `calendar list`.

### Requirement: pimdir queues scheduling intents
(backends) The refusal paragraph SHALL read: a reply to an event naming no `ORGANIZER` or no `ATTENDEE`, or a cancellation of one naming no `ATTENDEE`, SHALL be refused when its body is local, coded `invitation-missing`, the last pointing at `event delete`: an event naming no attendee is the account's own or invites nobody, and the account answers as one of its attendees. An event naming an object whose blob is absent SHALL refuse both, as `get_item` refuses its read, rather than queue them unchecked.

#### Scenario: A reply to the account's own event
- GIVEN a pimdir event whose `ORGANIZER` is the account and which names no `ATTENDEE`, as Graph writes one the account made
- WHEN `calendula --json pimdir reply <id> accept` runs
- THEN it exits 1 coded `invitation-missing` and nothing is queued

## REMOVED Requirements
