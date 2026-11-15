---
cairn: change
id: a-refused-item-is-coded
status: landed
created: 2026-11-15
---

# A refused item is coded

> Line K25 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), twin of cardamum's C29 (cardamum cairn/changes/archive/a-refused-or-missing-card-is-coded).

## Why

cardamum codes a vCard its server refuses `card-invalid`, so a script tells a card to fix from an account to grant. calendula had no counterpart: a CalDAV `PUT` refused for its iCalendar (`valid-calendar-data`, RFC 4791 §5.3.2.1) carried no code, and one refused with a 403 whose plain body names the iCalendar read `permission-denied`, the opposite of what to do. A Google event write answered 400 carried none either.

## What

- A new code, `item-invalid`, by pimalaya-cli's noun rule (cli cairn/spec/error.md "A code names the domain's noun": `item-` for calendars, `<noun>-invalid` as cardamum's `card-invalid` and Himalaya's `attachment-invalid`). No code of calendula's table or of pimalaya-cli's said it.
- **CalDAV**: one predicate, `rejects_item` (src/caldav/client.rs), read by the code reader and by a hint on `create_item` and `update_item`, so the two never disagree: a 400 or a 403 whose body names a precondition on the item's content (RFC 4791 §5.3.2.1: `supported-calendar-data`, `valid-calendar-data`, `valid-calendar-object-resource`, `supported-calendar-component`) or the iCalendar itself (`icalendar`, `vcalendar`, case ignored). A 403 naming `need-privileges` stays `permission-denied`; another precondition (`max-resource-size`, `min-date-time`...) keeps no code, as before. CardDAV defines two content preconditions, CalDAV four: the twin's rule maps to the four.
- **Google Calendar**: an event write (`events.insert`, `events.import`, `events.update`) answered 400 is `item-invalid`. calendula builds every parameter of that request (`sendUpdates` from a closed set), the calendar and the event id resolved before it, so a 400 there refuses the event sent ("a required field ... not provided, the value supplied is invalid, or the combination of provided fields is invalid", the Calendar API's errors guide). A 400 to any other request keeps no code.
- **Not covered: Graph.** Its 400 on a write does not name the body recognisably: `ErrorInvalidRequest` serves for many refusals, a quota (`ErrorQuotaExceeded`) and a malformed id (`ErrorInvalidIdMalformed`) come as 400 too, and the live campaign of 2026-11-15 observed no refused event body. vdir writes the bytes given and pimdir queues them: neither refuses a body.
- Spec: backends ("A server's refusal carries its code"), commands ("A JSON error carries a stable code").
