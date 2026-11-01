---
cairn: change
change: a-body-is-staged-under-the-lock
---

# Delta

## MODIFIED Requirements

### Requirement: pimdir writes are staged queue actions
A pimdir write SHALL append one action to the store's queue (pimdir STORAGE §15.1) through one producer opened for that write, before anything is written, and dropped after it: `create_item` to `add`, `update_item` to `update`, `delete_item` to `remove`.

The body SHALL reach the blob tree through that producer's blob writer, durably, before the row that pins it is appended, so the producer's staging lock (pimdir STORAGE §8) covers the body from its write to its enqueue and no collector can remove it in between. The action SHALL address the item by the public `seq` that is already the item's shared id.

`update_item` and `delete_item` SHALL honour `if_match` against the item's version. The engine still reconciles an applied edit against the base body it recorded at sync time: the precondition is the caller's, the merge the store's.

Because a queued create carries no public id until the owner applies it, `create_item` SHALL report the item's link id instead.
