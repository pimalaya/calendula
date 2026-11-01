---
cairn: change
id: a-write-is-checked-and-queued-in-one-step
status: landed
created: 2026-10-30
---

# A write is checked and queued in one step

## Why

io-pimdir landed what-the-app-ports-found, a-query-only-index and store-audit-2026-10-30 on 2026-10-30 (local checkout, path-pinned).

- **The gate runs twice on a body write.** `enqueue_with_body` (src/pimdir/backend.rs) queues through `PimdirProducer::enqueue_with_body`, which answers no partial support, then runs `check` again on the queued action to log them, once the body is in the tree for the gate to read. Run before the write, as cardamum did, that check reads no body and loses every body-dependent partial support (`calendar.scheduling`, `calendar.online-meeting`, `calendar.occurrence.update`). io-pimdir's `enqueue_with_body_checked` answers them as `PimdirEnqueued::partial`, read with the body in hand in the enqueue transaction.
- **The client's comments say a producer holds the staging lock while it lives** (src/pimdir/client.rs). io-pimdir's producer now holds it for its staging window alone, from its first staged body until a row pins it (store audit N8).
- Nothing else of wave 3c reaches calendula: no `PimdirBlobs::reader`, no drain report, no search, and `store-foreign` already maps (a reader now raises it for a `user_version` 0 database declaring tables). The drain now applies or parks an action on an item no source binds instead of skipping it; no calendula doc says otherwise.

## What

- `enqueue_with_body` goes through `enqueue_with_body_checked`, logs each partial support and answers them; the post-write `check` is gone. `delete_item` and the intents, carrying no body, keep `check` then `enqueue`.
- Test: a create into a calendar whose source supports `calendar.scheduling` in part reports that partial support, failing first on the check-then-enqueue order.
- The lock comments of src/pimdir/client.rs say what a producer holds now.
- `a_store_calendula_cannot_open_carries_its_code` also opens a store whose `user_version` is set to 0 and expects `store-foreign`, no word of deleting it.
- backends.md: the write requirement names `enqueue_with_body_checked` and says the partial supports are logged; the duplicate requirement names it.
