---
cairn: log
change: a-read-prints-its-list-row
landed: 2026-11-15
---

# A read prints its list row

An item prints as one row, `{id, calendarId, etag, contents, encoding}`, in `item list`, `item read` and `event read`, its bytes as text or as base64, never decoded lossily, and an event's absent `description` and `location` are `null`, lines K4 and K14 of the CLI alignment round. Capability **commands** moved.

- **src/shared/item.rs**: `CalendarItem` prints as `ItemRow` with the flattened `ItemContents`: `contents` the text and `encoding` `utf-8`, or the base64 and `base64`, both `null` with no bytes; base64-codec joins the dependencies, as in Himalaya and cardamum.
- **src/shared/item/read.rs, src/shared/event/read.rs**: the reads print the row (`event read` beside `events`), where `item read` printed `{contents, etag}` and `event read` `{contents, events}`, both decoded lossily; the text view reads undecodable bytes as U+FFFD.
- **src/shared/event.rs, src/shared/event/expand.rs, src/pimdir/index.rs**: `description` and `location` are optional.
- **Evidence**: on the code before, tests/read.rs read no `id` on `item read`, an integer array then a lossy string for a Latin-1 body, and `""` for an absent location; src/shared/event/build.rs read `""`.

CHANGELOG (Changed, breaking), MIGRATION.md. Spec: commands ("An item's bytes are text or base64", added; "The event projection", "A delete can be gated on a version", "Text output is plain text").
