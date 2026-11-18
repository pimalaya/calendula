---
cairn: log
change: a-missing-pimdir-body-is-pending
date: 2026-11-18
---

# A missing pimdir body is pending

Audit line TRI-3, the owner's ruling of 2026-11-18: a pimdir body the blob tree lacks is not hydrated and not an error (pimdir GUIDE §14), as Himalaya reads it. Capabilities **backends** and **commands** moved.

- **src/pimdir/backend.rs**: `item_from` takes io-pimdir's `None` for a blob gone or failing its name as no body, the item listed with no bytes and no `etag`, where it was an `item-unreadable` entry (`Body blob missing for item ...`) with the hash of the body it could not read, and exit 3. An item whose body failed with an I/O error keeps its entry and now reports no `etag` either. `body` fails such an item through `body_pending`, ``Item `<id>` in calendar `<calendar>` is not downloaded yet (body not fetched); run a sync to hydrate it``, coded `body-pending`, the failure `get_item` gives an unfetched item, so `item read`, `event read`, `event update`, `pimdir reply` and `pimdir cancel` answer it where they failed uncoded. `event delete`, which takes `body-pending` as the item being there, now deletes such an item.
- **Evidence**: `a_body_the_store_cannot_serve_is_an_entry_on_its_page` (the missing body no entry, no `etag`, `get_item` `body-pending` with its sentence) and `a_reply_or_a_cancel_over_a_missing_body_queues_nothing` (both coded `body-pending`) fail on the HEAD sources (an `etag` of the missing body; `code` `None`) and pass after.

CHANGELOG (Changed, breaking). MIGRATION (pimdir store). Spec: backends ("Each backend names the items it could not read", "pimdir is an availability-aware cache" and one scenario, "pimdir queues scheduling intents"), commands (the `item-unreadable` and `body-pending` rows). cardamum takes the twin change.
