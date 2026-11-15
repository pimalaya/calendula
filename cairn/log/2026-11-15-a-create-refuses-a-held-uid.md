---
cairn: log
change: a-create-refuses-a-held-uid
landed: 2026-11-15
---

# A create refuses a held UID

A gcal create naming a UID the calendar holds fails `item-duplicate` before anything is written, where `events.import` overwrote that event and exited 0, and a CalDAV create answered 412 carries `item-duplicate`, where it carried `precondition-failed`. Line K19, and I-F5 of K24, of the CLI alignment round, from the live testing campaign (G-F1, I-F5). Capabilities **backends** and **commands** moved.

- **src/gcal/backend.rs**: `holding` looks the UID up through `events.list?iCalUID=`, every page until one holds a match; `create_item` refuses a match, ``Calendar `<cal>` already holds `<uid>` as event `<id>`; update that event instead``, inserting and importing nothing.
- **src/caldav/backend.rs**: a 412 to a create is `item-duplicate`, ``Calendar `<cal>` already holds `<name>`; update that item instead``, the server's answer in `sources`.
- **Checked**: pimdir already refuses `item-duplicate`; Graph and vdir take a second item of one UID, which the code table now says.
- **Evidence**: `a_create_naming_a_held_uid_is_refused_and_imports_nothing` failed on the code before (the lookup's answer read as the import's, exit 0), and the 412 case of `a_duplicate_uid_and_a_stale_version_carry_their_code` read `precondition-failed`.

CHANGELOG (Changed), MIGRATION.md. Spec: backends ("A server's refusal carries its code", "Google Calendar backend"), commands (code table).
