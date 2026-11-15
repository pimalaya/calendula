---
cairn: change
id: a-write-receipt-says-queued
status: landed
created: 2026-11-15
---

# A write receipt says it is queued

> Line K3 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), findings F7 and F8, line A2 of its schema review; cardamum's C3 is the twin.

## Why

`event create`, `event update`, `item create` and `item update` printed `{id}`, their `queued` hidden by `#[serde(skip)]` ("which only the wording tells"), while their deletes print `{id, queued}`: a script could not tell a pimdir write waiting for a sync from one made. `calendar create` printed `{id, queued}`, where Himalaya's `mailbox create` prints `{id, name, queued}`. pimalaya-cli's output.md names `queued` as the receipt key for exactly this.

## What

- `event create`, `event update`, `item create` and `item update` print `{id, queued}` (their abandoned arm stays `{}`). A queued create's `id` stays the link id it was staged under, as Himalaya's `message add` prints the Message-ID it queued.
- `calendar create` prints `CalendarCreateOutput`, `{id, name, queued}`, `name` the display name given; its text is unchanged.
- Breaking for a script comparing the whole object: MIGRATION.md says so.
- Spec: commands ("Every data command returns a named output type").
