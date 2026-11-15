---
cairn: log
change: a-write-receipt-says-queued
landed: 2026-11-15
---

# A write receipt says it is queued

`event create`, `event update`, `item create` and `item update` print `{id, queued}` where `queued` was hidden, and `calendar create` prints `{id, name, queued}`, line K3 of the CLI alignment round. Capability **commands** moved.

- **src/shared/event/create.rs, src/shared/event/update.rs, src/shared/item/create.rs, src/shared/item/update.rs**: `queued` serialized; a queued create's `id` stays the link id it was staged under, as Himalaya's `message add` prints the Message-ID it queued.
- **src/shared/calendar/create.rs, src/json_schema.rs**: `CalendarCreateOutput`, registered under `calendula-calendar-create`; `Write::Create` goes.
- **Evidence**: tests/write.rs `every_write_receipt_says_whether_it_is_queued` read no `queued` and no `name` on the code before.

CHANGELOG (Changed, breaking), MIGRATION.md. Spec: commands ("Every data command returns a named output type").
