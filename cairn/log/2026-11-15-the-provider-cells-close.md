---
cairn: log
change: the-provider-cells-close
landed: 2026-11-15
---

# The provider cells close

The low findings of the live testing campaign, line K24 of the CLI alignment round (its I-F3 and I-F5 in a-calendar-update-clears-and-colours-first and a-create-refuses-a-held-uid). Capabilities **commands** and **backends** moved.

- **src/shared/event/fields.rs, src/shared/event/update.rs**: `event update` reads a zone-less `--start` or `--end` in the zone of the event's `DTSTART` (P2-F4), before the local anchoring of gcal and msgraph.
- **src/error.rs, src/pimdir/backend.rs**: `invitation-missing`; `pimdir reply` refuses an event naming no `ATTENDEE`, as Graph writes the account's own (P2-F7), and the reply and cancel refusals carry the code. calendula holds no address of the account, so an own event naming attendees still queues.
- **src/caldav/backend.rs**: a VTODO or a VJOURNAL is named after its UID (I-F4).
- **src/gcal/backend.rs**: a cancelled event reads `item-not-found` (G-F4); the primary calendar is the default (G-F7).
- **src/gcal/acl/update.rs**: `gcal acl update` reads scope and value from the rule id, a disagreeing `--scope` or `--value` refused `usage-invalid` before any request (G-F5).
- **src/msgraph/backend.rs**: an unset colour reads `null` (M-F2), and `isDefaultCalendar` names the default.
- **Evidence**: each test named in the change's tasks failed on the code before; the update test was checked by dropping the new call.

CHANGELOG (Added, Changed, Fixed). Spec: commands ("An update reads a zone-less time in the event's zone", added; field flags, gcal family, code table), backends (CalDAV, Google Calendar, Graph, pimdir intents).
