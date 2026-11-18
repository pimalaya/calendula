---
cairn: tasks
change: a-missing-pimdir-body-is-pending
---

# Tasks

- [x] src/pimdir/backend.rs: `item_from` reads a blob io-pimdir answers `None` for as no body, no entry, and reports an `etag` only for a body it read; `body` fails such an item through the new `body_pending`, which `get_item` shares for an unfetched one.
- [x] Tests `a_body_the_store_cannot_serve_is_an_entry_on_its_page` and `a_reply_or_a_cancel_over_a_missing_body_queues_nothing` rewritten, each seen failing on the HEAD sources; src/shared/item/list.rs fixture text.
- [x] Spec (backends, commands), CHANGELOG, MIGRATION.
