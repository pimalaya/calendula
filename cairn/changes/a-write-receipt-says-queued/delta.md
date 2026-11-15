---
cairn: delta
change: a-write-receipt-says-queued
---

# Delta

## ADDED Requirements

## MODIFIED Requirements

### Requirement: Every data command returns a named output type
(commands) The write arm of `event create`, `event update`, `item create` and `item update` SHALL print `{id, queued}`, `queued` whether a sync engine applies the write later, a queued create's `id` the link id it was staged under. `calendar create` SHALL print `{id, name, queued}`, `name` the display name it was given.

## REMOVED Requirements
