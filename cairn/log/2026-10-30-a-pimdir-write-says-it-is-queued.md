---
cairn: log
change: a-pimdir-write-says-it-is-queued
landed: 2026-10-30
---

# A pimdir write says it is queued

`CalendarClient::queues_writes` is true for pimdir alone, and the `create`, `update` and `delete` commands of the `event`, `todo`, `journal` and `item` families confirm a queued write as "`<Kind>` `<id>` queued for the next sync", "... update queued for the next sync" and "`<Kind>` deletion queued for the next sync". The JSON payloads are unchanged (the flag is `#[serde(skip)]`), so no field only one backend fills enters the shared outputs; a queued create's `id` stays the link id it staged, never a placeholder, per himalaya's 2026-08-27 queue-visibility decision. Closes C3 of cli-alignment-leftovers.

Test: `a_write_is_confirmed_as_queued` runs `event create`, `update` and `delete` over a pimdir store; with the old wording it failed on "Event `...` successfully created".

Capabilities moved: **backends** (a queued write is reported as queued, added; a create reports the identifier it was given, modified).
