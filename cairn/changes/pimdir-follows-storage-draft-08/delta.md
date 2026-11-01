---
cairn: change
change: pimdir-follows-storage-draft-08
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: pimdir is an availability-aware cache
An item whose body is not local (`level < Full`, no stored object) SHALL still list, carrying no bytes. `get_item` on such an item SHALL report a clear "body not fetched" state, the cue to sync, not a data-loss error.

A range filter SHALL still apply to it, read off the stored `text/calendar` summary rather than off bytes that are not local: a cache that hid its own undownloaded items from a date window would answer a different question than the one asked.

An item naming an object the blob tree holds no body for is a different state: the store is inconsistent rather than partially synced. The blob file may be absent, or its entry no body at all (a directory, a link, bytes that do not hash to its name), which the store reads as absent (pimdir STORAGE §5, §7). It SHALL still list, and the listing SHALL log a warning naming the item, since the row renders the same as an unhydrated one and `get_item` refuses it outright.

### Requirement: pimdir refuses a to-do where its calendar takes none
A `create_item` or `update_item` whose body is a VTODO SHALL be refused, and nothing queued, when a source concerned states `calendar.component.vtodo` `none` for the calendar, or source-wide with no row of the calendar overriding it (pimdir STORAGE §15.6, Annex B): for a create, a source syncing the calendar; for an update, a source binding the item. The capability is read only where a row states it, so a source stating nothing of it, declared or not, refuses no to-do. The error SHALL start `Stage the pimdir action:`, say the calendar takes no to-do, name the source, the capability and its stated reason when it gives one, and point at `-k/--calendar` to store the to-do in another calendar.

### Requirement: A pimdir store failure carries io-pimdir's code
A failure to open the store, or to stage into it, SHALL carry the stable code io-pimdir gives it (`PimdirError::code()`) when it is one a caller acts on: `store-uncreated`, `store-version`, `store-foreign`, `store-corrupt`, `store-hash-algo`, `store-busy`, `store-exposed`, `flag-invalid`, `action-malformed`. The message SHALL be io-pimdir's, but for `store-version`, and SHALL never advise deleting the store.

A store is never migrated (pimdir STORAGE §6): one of another format version than calendula's io-pimdir is refused as `store-version`, whichever way it differs. One naming no format version, or an earlier draft's, SHALL be refused with a message saying the store was written by an earlier sync engine, and giving the reinstall in the order that keeps its queued writes: sync it with the neverest that wrote it until nothing is queued, then reinstall it with `neverest sync --reset --force -a <account>`, naming the account neverest syncs it for, which moves it aside, deleting nothing, and syncs a new one. Any other SHALL be refused with a message saying a newer sync engine wrote it and to upgrade calendula.

### Requirement: pimdir queues scheduling intents
`pimdir reply <EVENT-ID> accept|tentative|decline` SHALL append one `calendar-reply` action `{ "v": 1, "source", "seq", "partstat": "ACCEPTED"|"TENTATIVE"|"DECLINED", "comment"?, "recurrence_id"? }`, and `pimdir cancel <EVENT-ID>` one `calendar-cancel` action `{ "v": 1, "source", "seq", "comment"?, "recurrence_id"? }` (pimdir STORAGE Annex B.2), built and validated by io-pimdir's `PimdirInvitation`, anchored on the item's calendar, through the producer and its §15.6 gate, and SHALL touch no item.

A payload its schema refuses SHALL be refused before anything is queued: a `partstat` outside the three, an item named by both or neither of `seq` and `link_id`, a cancel stating `partstat`, an empty `source`. calendula names the item by its `seq` alone, a reply's `partstat` from its answer and a cancel's none; a refusal io-pimdir raises carries `action-malformed`.

`source` SHALL name the performer in every payload. In a calendar a source syncing it declares capabilities for, it SHALL be `--source` when given, else the single candidate, else the recorded choice; several candidates and no choice SHALL be refused, listing them. In a calendar whose sources declare nothing, it SHALL be `--source` when given, else the one source syncing the calendar; several sources SHALL be refused, listing them, and none refused, both pointing at `--source`, nothing queued.

`-r/--recurrence-id` SHALL limit either to one occurrence, carried as `recurrence_id`, spelled as the `recurrenceId` an expanded listing shows (the series' `DTSTART` form). When the body is local, a value naming no occurrence of the series, or naming one in another spelling, SHALL be refused. A performer not declaring `calendar.reply.occurrence` or `calendar.cancel.occurrence`, and a store declaring nothing, SHALL be refused with the code `occurrence-unsupported` and an error starting `Occurrence not supported:`, naming the capability and the source, or saying no source of the store declares it; nothing is queued.

A reply to an event naming no `ORGANIZER`, or a cancellation of one naming no `ATTENDEE`, SHALL be refused when its body is local, the latter pointing at `event delete`. An event naming an object whose blob is absent SHALL refuse both, as `get_item` refuses its read, rather than queue them unchecked.

### Requirement: A JSON error carries a stable code
A failure a caller is expected to act on SHALL carry a stable code, printed under `--json` as a `code` string beside `error`, `sources` and `backtrace`. The code SHALL be found anywhere in the error chain, so added context does not hide it. A failure with no code SHALL print no `code` field. The plain output SHALL stay unchanged. Codes are kebab-case and never renamed; `error` stays free wording.

| Code | Raised when |
|---|---|
| `body-pending` | a listed item's body is not local yet (pimdir); a sync brings it |
| `precondition-failed` | a write's `--if-match` names a version the item no longer has (pimdir) |
| `occurrence-unsupported` | `pimdir reply` or `pimdir cancel` names an occurrence its performer cannot act on alone |
| `item-duplicate` | a pimdir create names a `UID` its calendar holds or has queued |
| `store-uncreated` | the pimdir store has no schema yet: its sync engine creates it |
| `store-version` | the pimdir store is of another format version than calendula's: an earlier sync engine's is reinstalled, a newer one's needs calendula upgraded |
| `store-foreign` | the pimdir database is not a pimdir store: another program wrote it |
| `store-corrupt` | the pimdir store's two version stamps disagree, or its schema is not the canonical one |
| `store-hash-algo` | the pimdir store names its bodies by a hash this calendula does not know: upgrade calendula |
| `store-busy` | another process holds the pimdir store; retry |
| `store-exposed` | a path of the pimdir store is a link or another user can reach it: restrict it |
| `flag-invalid` | a pimdir write carries a flag the store cannot hold |
| `action-malformed` | a pimdir queue action's payload is one its schema refuses |

#### Scenario: A stale version
- GIVEN a pimdir event whose body changed since it was read
- WHEN `calendula --json event delete --if-match <old etag>` names it
- THEN the command exits 1, queues nothing and prints `{"code":"precondition-failed","error":"Precondition failed: …","sources":[],"backtrace":null}`

## REMOVED Requirements

None. The codes `store-newer` and `store-stale`, added since 0.2.0 and never released, leave the table above with the failures they named: io-pimdir raises neither, a store of another format version being one failure, `store-version`.
