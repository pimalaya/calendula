---
cairn: delta
id: an-event-owes-a-dtstart
---

# Delta

## MODIFIED Requirements

### Requirement: A composed item is checked before it is written
What the composer wrote SHALL be checked against the RFC 5545 contract through ical-rs's validator rather than a look at its first line. Reading is liberal and this is the strict half: a VCALENDAR missing its required PRODID, a VEVENT missing its UID or DTSTAMP, are caught here rather than by the server or by nobody.

A VEVENT SHALL also be refused when it carries no DTSTART and the calendar specifies no METHOD (RFC 5545 3.6.1). That requirement is a condition on the enclosing calendar rather than a property list, so ical-rs's per-component contract cannot state it; calendula SHALL state it, because it is the one thing a person writing an event by hand leaves out and because a server does worse than refuse it: SabreDAV denormalizes DTSTART into its index on write and answers HTTP 500.

An item that does not pass SHALL have its violations printed and SHALL offer to re-open the editor, defaulting to yes. This is not a menu: the only question is whether to fix it, and declining is an error rather than an abandon.

An iCalendar given on the command line SHALL NOT be checked, going to the backend as it was written: that is the promise the projections already make, calendula never rewriting bytes it was handed.

### Requirement: A minted item carries an identity, and its family names its kind
An item minted from nothing SHALL be a VCALENDAR carrying a PRODID and one component of the family's own kind, carrying a UID minted as a fresh UUID and a DTSTAMP of now. An editor handed an empty file mints no identity, and the CalDAV resource name and the pimdir link id both derive from the UID.

A minted VEVENT SHALL also carry a DTSTART of now, so the seed passes the check it will be held to the moment the composer hands it back. The value is a placeholder as the identity is, and an event with no time is not an event. A VTODO and a VJOURNAL owe none and SHALL carry none.

`item` names no component kind, so it SHALL mint nothing: `item build` and `item create` given no source SHALL bail, naming `event`, `todo` and `journal` as the families that can.
