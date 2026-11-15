---
cairn: delta
change: pimdir-adopts-the-io-pimdir-release-api
---

# Delta

## ADDED Requirements

### Requirement: pimdir refuses a create whose UID the calendar holds
A `create_item` whose link id (its `UID` under the store's derivation, pimdir STORAGE Annex A.3) names an item the calendar holds live, or a create already queued there, SHALL be refused through io-pimdir's `enqueue_with_body`, whose duplicate check, body write and append are one transaction, and nothing queued nor written. The error SHALL carry the code `item-duplicate`, name the calendar and the link id, and name the live item by its id, to be updated instead, or the queued create by its queue row.

### Requirement: A pimdir store failure carries io-pimdir's code
A failure to open the store, or to stage into it, SHALL carry the stable code io-pimdir gives it (`PimdirError::code()`) when it is one a caller acts on: `store-uncreated`, `store-newer`, `store-foreign`, `store-stale`, `store-busy`. A store older than calendula's schema SHALL be read as it stands, failing nothing. The message SHALL be io-pimdir's, which says to run or upgrade the store's sync engine, or to upgrade this program for a newer store, and never to delete the store.

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
| `store-busy` | another process holds the pimdir store; retry |

#### Scenario: A stale version
- GIVEN a pimdir event whose body changed since it was read
- WHEN `calendula --json event delete --if-match <old etag>` names it
- THEN the command exits 1, queues nothing and prints `{"code":"precondition-failed","error":"Precondition failed: …","sources":[],"backtrace":null}`

### Requirement: pimdir writes are staged queue actions
A pimdir write SHALL append one action to the store's queue (pimdir STORAGE §15.1) through one producer opened for that write, before anything is written, and dropped after it: `create_item` to `add`, `update_item` to `update`, `delete_item` to `remove`.

A body SHALL be written with the row that pins it in one step, through that producer's `enqueue_with_body`, which hashes it under the store's algorithm, runs the store's gate (pimdir STORAGE §15.6) and, for an `add`, the duplicate check on the bytes in hand, then writes the body durably and appends the row in one transaction under the producer's staging lock (§8): no collector can remove the body before its row pins it, and a refused write leaves no body in the tree. calendula SHALL NOT hash or write a body itself. The action SHALL address the item by the public `seq` that is already the item's shared id.

`update_item` and `delete_item` SHALL honour `if_match` against the item's version. The engine still reconciles an applied edit against the base body it recorded at sync time: the precondition is the caller's, the merge the store's.

Because a queued create carries no public id until the owner applies it, `create_item` SHALL report the item's link id instead.

### Requirement: pimdir queues scheduling intents
`pimdir reply <EVENT-ID> accept|tentative|decline` SHALL append one `calendar-reply` action `{ "v": 1, "source"?, "seq", "partstat": "ACCEPTED"|"TENTATIVE"|"DECLINED", "comment"?, "recurrence_id"? }`, and `pimdir cancel <EVENT-ID>` one `calendar-cancel` action `{ "v": 1, "source"?, "seq", "comment"?, "recurrence_id"? }` (pimdir STORAGE Annex B.2), built by io-pimdir's `PimdirInvitation`, anchored on the item's calendar, through the producer and its §15.6 gate, and SHALL touch no item.

`source` SHALL name the performer: `--source` when given, else the single candidate, else the recorded choice; several candidates and no choice SHALL be refused, listing them. A store whose sources declare nothing SHALL get no `source` unless one is given.

`-r/--recurrence-id` SHALL limit either to one occurrence, carried as `recurrence_id`, spelled as the `recurrenceId` an expanded listing shows (the series' `DTSTART` form). When the body is local, a value naming no occurrence of the series, or naming one in another spelling, SHALL be refused. A performer not declaring `calendar.reply.occurrence` or `calendar.cancel.occurrence`, and a store declaring nothing, SHALL be refused with the code `occurrence-unsupported` and an error starting `Occurrence not supported:`, naming the capability and the source; nothing is queued.

A reply to an event naming no `ORGANIZER`, or a cancellation of one naming no `ATTENDEE`, SHALL be refused when its body is local, the latter pointing at `event delete`. An event naming an object whose blob is absent SHALL refuse both, as `get_item` refuses its read, rather than queue them unchecked.

### Requirement: Local backend auto-detected
A typed folder path or `file://` URL SHALL configure a local backend, auto-detecting the kind from on-disk markers: a pimdir index file (`pimdir.db`) means pimdir, a directory holding at least one collection (a subdirectory carrying an `.ics` file or a vdir metadata marker) means vdir. A blob directory without its index SHALL NOT mean pimdir: no reader opens a store its sync engine has not created.

pimdir SHALL be tested first, since a store also holds subdirectories and testing vdir first would misread every store as a home.

The wizard SHALL prompt vdir-against-pimdir only when both backends are compiled in and detection is inconclusive, which an empty directory is.
