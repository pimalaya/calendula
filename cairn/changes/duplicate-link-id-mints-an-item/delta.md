---
cairn: change
change: duplicate-link-id-mints-an-item
---

# Delta

## ADDED Requirements

### Requirement: A UID is not an address
The pimdir backend SHALL NOT assume an item's link id is the `UID` its body carries, nor that a `UID` identifies at most one item in a calendar. A store may hold two calendar object resources of one calendar sharing a `UID`, keyed apart by the store (pimdir SPEC §9), and both SHALL list, read and act as ordinary items, addressed by their own public `seq`.

What stays unique is the key and the public id: `(collection, link_id)` still names one item and `seq` still names one resource. What ends is the link id being derivable from the body, so a read that re-derives a `UID` in order to address a row is addressing an unknown number of them.

RFC 4791 §4.1 requires the `UID` to be unique in the collection and servers do not always enforce it. The two copies need not even be the same event: a verified case held two different meetings under one `UID`. Resolving an identity to whichever row came first would hide one of them.

This is a second **resource**, not a second component: every component sharing a `UID` still lives in one resource, so each of the two items holds its own whole recurrence set.

#### Scenario: A duplicated event lists twice
- GIVEN a calendar whose store holds two items whose bodies carry one `UID`
- WHEN the backend lists it
- THEN both appear, each with its own public id, and neither is marked

## MODIFIED Requirements

### Requirement: pimdir shows a short public id
The pimdir backend SHALL show and accept each item's public id (`items.seq`, a small store-assigned integer stable across every collection the item is filed in), not the internal `link_id`. It SHALL resolve that id to the `link_id` before reading a body or staging a change, and SHALL fail clearly on a non-numeric id rather than looking up nothing.

Addressing by the public id is what keeps two duplicated resources distinguishable: they carry one `UID` between them and have two `seq`s, so an address derived from the body would be ambiguous where a `seq` is not.

## REMOVED Requirements
