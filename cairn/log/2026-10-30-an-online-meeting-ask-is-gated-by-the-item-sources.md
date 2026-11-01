---
cairn: log
change: an-online-meeting-ask-is-gated-by-the-item-sources
landed: 2026-10-30
---

# An online-meeting ask on an update is gated by the item's sources

`check_online_meeting` reads `item_capabilities` for an `update`, the sources binding the item, and the collection's `capabilities` for an `add`, as the store's own gate does (STORAGE §15.6).

Test: `an_online_meeting_on_an_update_is_gated_by_the_item_sources` gives a second source a checkpoint on the calendar and a full declaration, leaving the item bound to an undeclared one; with the collection read the update was queued (`unwrap_err` on `Ok`), now it is refused and nothing is queued.

Capabilities moved: **backends** (pimdir refuses an online meeting no source can create, modified).
