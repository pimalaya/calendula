---
cairn: delta
change: a-stale-pimdir-edit-is-parked
---

# Delta

## ADDED Requirements

### Requirement: A queued pimdir edit names the body it replaced
An `update` that `update_item` queues SHALL state as its `if_match` (pimdir STORAGE §15.3) the item's body hash when the write was gated on a version, by `--if-match` or by the read an update starting from the stored item makes, and none when the caller read no body. The store's owner then parks, rather than applies, an edit whose item holds another body by the time it drains, and the parked row SHALL show in `pimdir status` as every parked action does.

#### Scenario: A sync lands first
- GIVEN an event read at version `A` and an `event update` queued from it
- WHEN a sync replaces the event's body with `B` before the queue drains
- THEN the owner parks the update, the event keeps `B`, and `pimdir status` lists the row with an error naming `A` and `B`
