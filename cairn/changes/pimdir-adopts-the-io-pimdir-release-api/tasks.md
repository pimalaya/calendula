---
cairn: tasks
change: pimdir-adopts-the-io-pimdir-release-api
---

# Tasks

- [x] io-pimdir `stage_body`, `PimdirError::code()`, `enqueue_add`: built against the local checkout (`[patch.crates-io]` `path = "../io-pimdir"`, a NOTE saying it is temporary) until those changes are pushed; Cargo.lock moved the io-pimdir entry alone.
- [x] Store errors: codes from `PimdirError::code()` (src/pimdir/client.rs `store_error`, vocabulary in src/error.rs), the message io-pimdir's, pointing at the owning sync engine, never "delete the store"; `a_store_calendula_cannot_open_carries_its_code` covers an uncreated and a newer store. A stale and a busy store are not built by the test (no SQL in calendula; busy needs a 30 s SQLite timeout): their mapping is the same match. io-pimdir's store version contract (2026-10-30): `store-foreign` added, `store-older` dropped (a reader and a producer read an older store as it stands, so it is never raised); a foreign database is not built by the test either (it needs a full schema with another `store_meta.format`, so SQL). The new `PimdirError::Queued` and `Role` come from owner verbs calendula never calls; every match on `PimdirError` keeps a wildcard arm.
- [x] Duplicate creates refused through `enqueue_add` (live link id, pending create) with `item-duplicate`; `a_create_whose_uid_is_live_or_queued_is_refused` failed on the old code (`unwrap_err()` on `Ok("alone@example.org")`).
- [x] Bodies staged with their row in one step through `enqueue_with_body` (first through `stage_body`); calendula no longer hashes or writes a body, and the `stage_body` and `enqueue` helpers of src/pimdir/backend.rs are gone.
- [x] A refused write leaves no blob: `create_item` and `update_item` go through io-pimdir's `enqueue_with_body`, the gate and the duplicate check run on the bytes in hand and the body written in the transaction appending its row. `a_refused_create_leaves_no_body_in_the_tree` failed on the old code (objects/ held one more file after a refused duplicate create). The partial supports a write passes with are read by running the gate again once it is queued, `enqueue_with_body` answering the row alone.
- [x] A missing blob turns `reply` and `cancel` into an error rather than skipping their checks (src/pimdir/backend.rs `body`, shared with `get_item`); `a_reply_or_a_cancel_over_a_missing_body_queues_nothing` failed on the old code (`unwrap_err()` on `Ok(PimdirQueued { row: 2, seq: 1, .. })`).
- [x] The wizard requires pimdir.db, not just objects/ (src/wizard/local.rs); `a_blob_tree_alone_marks_no_pimdir_store` failed on the old code.
- [x] Draft citations in CHANGELOG.md: `draft-03`, `draft-04` and `ef8eae0` replaced by the spec documents and sections (pimdir STORAGE §14, §15.6, Annex B.2).
- [x] Release: io-pimdir 0.10 from crates.io, the `[patch.crates-io]` pins dropped, Cargo.lock refreshed, the CHANGELOG io-pimdir line naming 0.10 (done by published-dependencies, 2026-11-12; the store drafts the CHANGELOG names are storage-draft-09, sync-draft-08 and search-draft-09 since pimdir-follows-storage-draft-09).
- [x] CHANGELOG entries.
- [x] Delta folded into [backends](../../spec/backends.md), [commands](../../spec/commands.md) and [wizard](../../spec/wizard.md) on 2026-10-30, the code behaving so; at landing only the log entry and the status remain.
- [x] Log; land.
