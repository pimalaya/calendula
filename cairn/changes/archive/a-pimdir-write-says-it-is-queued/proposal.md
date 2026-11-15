---
cairn: change
id: a-pimdir-write-says-it-is-queued
status: landed
created: 2026-10-30
---

# A pimdir write says it is queued

## Why

A pimdir write only appends an action to the store's queue; the owner applies it and a sync pushes it later, or parks it. The shared confirmations still read "Event `<id>` successfully created", "updated", "deleted", as if the server had it (cli-alignment-leftovers C3, review of 2026-10-30). A create also reports the link id it staged, which no verb accepts until the owner applies the action, and the confirmation does not say so.

The decided rule for queued creations (himalaya/cairn/log/2026-08-27-pimdir-queue-visibility.md, confirmed 2026-10-30): no placeholder id (no `q<id>`, `0`, `""`); a create returns the link id it staged; a queue view shows the row id as ROW; `null` is the id wherever a listing shows a queued creation. calendula lists no queued creation and has no queue view, so the create's report is what this change touches.

## What

- The shared client tells whether a backend queues its writes for a sync rather than making them on the server; only pimdir does.
- `event`, `todo`, `journal` and `item` `create`, `update` and `delete` word a queued write as queued: "Event `<id>` queued for the next sync", "Event `<id>` update queued for the next sync", "Event deletion queued for the next sync". Every other backend keeps its wording.
- The JSON payloads do not change: `{"id"}` stays the link id on pimdir, documented as an id no command accepts until a sync applies the create.

## Out

- A `queued` field in the JSON payloads: one backend alone would set it, against "Shared outputs carry no backend details"; revisit if a caller needs it.
- Refusing a create whose `UID` is already live or already queued: change pimdir-adopts-the-io-pimdir-release-api.
