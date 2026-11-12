---
cairn: log
change: pimdir-adopts-the-io-pimdir-release-api
landed: 2026-11-12
---

# The pimdir backend adopts the io-pimdir release API

The code landed on 2026-11-01 (cc34e87) against an unreleased io-pimdir; the release task closes now that calendula takes io-pimdir 0.10 from crates.io (published-dependencies, the same day).

- Store errors carry io-pimdir's codes (`store-uncreated`, `store-newer`, `store-foreign`, `store-stale`, `store-busy`, through `PimdirError::code()` in src/pimdir/client.rs `store_error`), with io-pimdir's message, which names the sync engine to run or upgrade and never says to delete the store.
- A create whose `UID` is live in the calendar or already queued there is refused as `item-duplicate` through io-pimdir's `enqueue_add`.
- A body is staged with the row pinning it through `PimdirProducer::enqueue_with_body`, so a refused write leaves no blob; calendula no longer hashes or writes bodies.
- A missing blob fails `pimdir reply` and `pimdir cancel` as it fails `event read`.
- The wizard takes a folder for a store only when it holds pimdir.db.

Tests (each seen failing on the old code): `a_store_calendula_cannot_open_carries_its_code`, `a_create_whose_uid_is_live_or_queued_is_refused`, `a_refused_create_leaves_no_body_in_the_tree`, `a_reply_or_a_cancel_over_a_missing_body_queues_nothing`, `a_blob_tree_alone_marks_no_pimdir_store`.

Capabilities moved (folded on 2026-10-30): backends, commands, wizard.
