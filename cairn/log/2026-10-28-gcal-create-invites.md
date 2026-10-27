---
cairn: log
change: gcal-create-invites
landed: 2026-10-28
---

# A gcal event created with attendees invites them

`GcalBackend::create_item` in src/gcal/backend.rs sends an event through `events.insert` with `sendUpdates=all`, its UID as the `iCalUID`, when `invites` holds: the event names an `ATTENDEE` and either no `ORGANIZER` or one whose address is the calendar id, compared case-insensitively. Every other event goes through `events.import` (with a UID) or a plain `events.insert` (without one), as before, so an invitation from someone else is still stored without mailing its guests.

Verification: `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` with all features; checked by hand against a Google calendar, a second mailbox receiving the invitation both with no `ORGANIZER` and with the account as `ORGANIZER`.

Capabilities moved: **backends** (gcal invites the attendees of an event it organizes, modified).
