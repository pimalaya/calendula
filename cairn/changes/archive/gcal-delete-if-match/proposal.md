---
cairn: change
id: gcal-delete-if-match
status: landed
created: 2026-10-30
---

# A gcal delete honours `--if-match`

## Why

`event delete --if-match` (and the `todo`, `journal` and `item` deletes) is refused on gcal: when the flag landed, the delete went through io-gcal with no precondition, so refusing it was better than dropping it. io-gcal 0.2.0's `event_delete` now takes an `if_match` and sends it as `If-Match`, the way `event_update` already does, so the refusal only hides a guard Google offers. Raised in #21.

## What

`GcalBackend::delete_item` passes `if_match` to `event_delete` instead of refusing it. A version Google no longer has fails with its `412 Precondition Failed`, as a gated update already does. vdir, which has no entity tag, keeps refusing the flag.

Out of scope: mapping Google's 412 to the `precondition-failed` code, which no gcal write carries yet, update included.
