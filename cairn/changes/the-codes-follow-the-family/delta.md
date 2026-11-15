---
cairn: delta
change: the-codes-follow-the-family
---

# Delta

## ADDED Requirements

## MODIFIED Requirements

### Requirement: Calendar selection
(commands) A unified or pimdir command naming no calendar, on an account with no `calendar.default`, SHALL fail coded `calendar-default-missing`, ``No default calendar found, name one with `-k <CALENDAR-ID>` or set `calendar.default` ``.

### Requirement: A JSON error carries a stable code
(commands) The code table gains `calendar-default-missing`, `store-not-current`, `sync-version`, `store-owned`, `store-staging`, `store-in-flight`, `store-reset`, `conflict-unread`, `unsupported`, `no-performer` and `ambiguous-performer`.

### Requirement: A pimdir store failure carries io-pimdir's code
(backends) A store failure SHALL carry the code io-pimdir gives it when it is one Himalaya relays, by the same spelling: `store-version`, `store-uncreated`, `store-foreign`, `store-corrupt`, `store-not-current`, `store-hash-algo`, `sync-version`, `store-busy`, `store-owned`, `store-staging`, `store-in-flight`, `store-exposed`, `store-reset`, `lock-replaced`, `item-duplicate`, `flag-invalid`, `action-malformed`, `conflict-unread`, `unsupported`, `no-performer`, `ambiguous-performer` and the index and query codes; not `tag-refused`, which only a mail tag raises. A refusal calendula words itself SHALL keep the code: a write a source does not support `unsupported`, an intent no source syncs `no-performer`, several `ambiguous-performer`.

## REMOVED Requirements
