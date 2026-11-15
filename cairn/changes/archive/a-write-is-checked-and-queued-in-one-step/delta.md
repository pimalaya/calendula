---
cairn: change
change: a-write-is-checked-and-queued-in-one-step
---

# Delta

## MODIFIED Requirements

### Requirement: pimdir writes are staged queue actions
A pimdir write SHALL append one action to the store's queue (pimdir STORAGE §15.1) through one producer opened for that write, before anything is written, and dropped after it: `create_item` to `add`, `update_item` to `update`, `delete_item` to `remove`.

A body SHALL be written with the row that pins it in one step, through that producer's `enqueue_with_body_checked`, which hashes it under the store's algorithm, runs the store's gate (pimdir STORAGE §15.6) and, for an `add`, the duplicate check on the bytes in hand, then writes the body durably and appends the row in one transaction under the producer's staging lock (§8): no collector can remove the body before its row pins it, and a refused write leaves no body in the tree. The partial supports the gate passed the action with, those the body asks for (`calendar.scheduling`, `calendar.online-meeting`, `calendar.occurrence.update`) included, SHALL each be logged as a warning. calendula SHALL NOT hash or write a body itself. The action SHALL address the item by the public `seq` that is already the item's shared id.

`update_item` and `delete_item` SHALL honour `if_match` against the item's version. The engine still reconciles an applied edit against the base body it recorded at sync time: the precondition is the caller's, the merge the store's.

Because a queued create carries no public id until the owner applies it, `create_item` SHALL report the item's link id instead.

### Requirement: pimdir refuses a create whose UID the calendar holds
A `create_item` whose link id (its `UID` under the store's derivation, pimdir STORAGE Annex A.3) names an item the calendar holds live, or a create already queued there, SHALL be refused through io-pimdir's `enqueue_with_body_checked`, whose duplicate check, body write and append are one transaction, and nothing queued nor written. The error SHALL carry the code `item-duplicate`, name the calendar and the link id, and name the live item by its id, to be updated instead, or the queued create by its queue row.
