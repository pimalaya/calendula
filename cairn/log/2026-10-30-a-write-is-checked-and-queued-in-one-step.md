---
cairn: log
change: a-write-is-checked-and-queued-in-one-step
landed: 2026-10-30
---

# A write is checked and queued in one step

calendula builds against io-pimdir's what-the-app-ports-found, a-query-only-index and store-audit-2026-10-30 (2026-10-30, the local checkout still path-pinned; Cargo.lock unchanged).

`enqueue_with_body` in src/pimdir/backend.rs queues through `PimdirProducer::enqueue_with_body_checked`, logs each partial support of the answered `PimdirEnqueued::partial` and answers them; the `check` it ran again after the write, for the partial supports the enqueue did not answer, is gone, so the gate runs once. `delete_item` and the intents, carrying no body, keep `check` then `enqueue`. The comments of src/pimdir/client.rs say a producer holds the staging lock from the body it stages until the row pinning it commits, io-pimdir's producer no longer holding it while idle (store audit N8).

Nothing else of wave 3c reaches calendula: no `PimdirBlobs::reader`, no drain report, no search, and `store-foreign`, which a reader now raises for a `user_version` 0 database declaring tables, was already mapped. No calendula doc said a queued action on an item no source binds is skipped, so none changed.

Test: `a_partial_support_the_body_asks_for_is_reported` creates a scheduling invitation where `caldav` supports `calendar.scheduling` in part and asserts that partial support is answered. With the gate run before the write (the check-then-enqueue order) it failed (`left: []`), the gate reading no body; the post-write re-check calendula had used passed it too, so no partial support was lost before this change, only gated twice. `a_store_calendula_cannot_open_carries_its_code` also sets a real store's `user_version` to 0 and expects `store-foreign` with no word of deleting it (io-pimdir answered `store-uncreated` there before). 227 tests pass with every feature and with the default ones, 184 with pimdir alone; clippy is clean with every feature, the default ones, and each backend alone with `rustls-ring`.

Capabilities moved: **backends** (pimdir writes are staged queue actions, pimdir refuses a create whose UID the calendar holds, modified).
