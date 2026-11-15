---
cairn: delta
change: a-damaged-store-carries-its-code
---

## MODIFIED Requirements

### Requirement: A JSON error carries a stable code
A failure a caller is expected to act on SHALL carry a stable code, printed under `--json` as a `code` string beside `error`, `sources` and `backtrace`. The code SHALL be found anywhere in the error chain, so added context does not hide it. A failure with no code SHALL print no `code` field. The plain output SHALL stay unchanged. Codes are kebab-case and never renamed; `error` stays free wording.

| Code | Raised when |
|---|---|
| `body-pending` | a listed item's body is not local yet (pimdir); a sync brings it |
| `precondition-failed` | a write's `--if-match` names a version the item no longer has (pimdir) |
| `occurrence-unsupported` | `pimdir reply` or `pimdir cancel` names an occurrence its performer cannot act on alone |
| `item-duplicate` | a pimdir create names a `UID` its calendar holds or has queued |
| `store-uncreated` | the pimdir store has no schema yet: its sync engine creates it |
| `store-newer` | the pimdir store's schema is newer than calendula's: upgrade calendula |
| `store-foreign` | the pimdir database is not a pimdir store: another program wrote it |
| `store-stale` | the pimdir store was written by an earlier draft: run or upgrade its sync engine |
| `store-corrupt` | the pimdir store's two version stamps disagree |
| `store-hash-algo` | the pimdir store names its bodies by a hash this calendula does not know: upgrade calendula |
| `store-busy` | another process holds the pimdir store; retry |

#### Scenario: A stale version
- GIVEN a pimdir event whose body changed since it was read
- WHEN `calendula --json event delete --if-match <old etag>` names it
- THEN the command exits 1, queues nothing and prints `{"code":"precondition-failed","error":"Precondition failed: …","sources":[],"backtrace":null}`

### Requirement: A pimdir store failure carries io-pimdir's code
A failure to open the store, or to stage into it, SHALL carry the stable code io-pimdir gives it (`PimdirError::code()`) when it is one a caller acts on: `store-uncreated`, `store-newer`, `store-foreign`, `store-stale`, `store-corrupt`, `store-hash-algo`, `store-busy`. A store older than calendula's schema SHALL be read as it stands, failing nothing. The message SHALL be io-pimdir's, which says to run or upgrade the store's sync engine, or to upgrade this program for a newer store, and never to delete the store.

