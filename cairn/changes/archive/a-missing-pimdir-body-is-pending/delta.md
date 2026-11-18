---
cairn: delta
change: a-missing-pimdir-body-is-pending
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: Each backend names the items it could not read (backends)

pimdir: a body the blob tree no longer holds is no entry, the item listed as an unfetched one (pimdir GUIDE §14); a body that cannot be read stays an `item-unreadable` entry.

### Requirement: pimdir is an availability-aware cache (backends)

An item whose body is not local, never fetched or its blob file gone or failing its name, lists with no bytes and no `etag`, and `get_item` fails it coded `body-pending` with the unfetched sentence, exit 1, as Himalaya. Only a body the store fails to read is an `item-unreadable` entry. One scenario added.

### Requirement: pimdir queues scheduling intents (backends)

An event whose blob is absent refuses both coded `body-pending`.

### Requirement: A JSON error carries a stable code (commands)

`item-unreadable` no longer covers a body missing from the store; `body-pending` is the failure of `item read` or `event read` on an item whose body is not local, either way.

## REMOVED Requirements

None.
