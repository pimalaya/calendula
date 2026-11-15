---
cairn: log
change: an-event-is-named-by-its-id
landed: 2026-11-15
---

# An event is named by its id

`event delete` takes the event's id alone, as `event read` and `event update` do, a `UID` going through `event search`, line K15 of the CLI alignment round. No backend but pimdir resolves a `UID` short of listing the whole calendar, so the answer to "every verb or none" is none. Capability **commands** moved.

- **src/shared/event/delete.rs**: the listing fallback and its `UID` match go; the id is read first, a miss coded `item-not-found` naming `event search <UID>`.
- **src/mcp.rs**: the instructions say the event tools take no UID.
- **Evidence**: on the binary before, tests/event_list.rs `a_uid_is_no_event_id` failed (the event deleted by its `UID`), and the msgraph test before sent a second request, the listing.

CHANGELOG (Changed, breaking), MIGRATION.md. Spec: commands ("An event is named by its id alone" replaces "An event is deleted by its id or its UID").
