---
cairn: change
change: a-pimdir-write-says-it-is-queued
---

# Delta

## ADDED Requirements

### Requirement: A queued write is reported as queued
A backend whose writes are queued for a sync to push, rather than made on the server, SHALL have its writes confirmed as queued: `create` as "`<Kind>` `<id>` queued for the next sync", `update` as "`<Kind>` `<id>` update queued for the next sync", `delete` as "`<Kind>` deletion queued for the next sync", for the `event`, `todo`, `journal` and `item` families. Every other backend SHALL keep "successfully created", "updated", "deleted". The JSON payloads SHALL be the same for every backend.

pimdir is the only backend that queues its writes.

## MODIFIED Requirements

### Requirement: A create reports the identifier it was given
`create_calendar` and `create_item` SHALL return the identifier the backend actually assigned, and the command SHALL report that one.

It is the requested id on every backend that lets a client name a collection, and a server-minted one where it does not, so a create never names a resource that does not exist. A queued create has no identifier yet: it SHALL report the link id it staged, never a placeholder (no `q<id>`, `0` or empty string), and no command accepts that id until a sync applies the create.
