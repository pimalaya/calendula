---
cairn: delta
change: the-queue-is-read-and-retracted
---

# Delta

## ADDED Requirements

### Requirement: pimdir reads and retracts its queue
(backends) `pimdir queue list`, `pimdir queue show <ROW>` and `pimdir queue cancel <ROW>` SHALL serve the store's queue as Himalaya's do. `queue list` (`ls`) SHALL list the calendar `-k/--calendar` names, refused `calendar-not-found` when the account holds none of that id, or every calendar of the account when it is omitted, as cardamum's lists its address books, and print `{items, parked}`: each creation and each intent waiting, the rows no listing shows (an update or a removal addresses an item a listing shows), as `{row, queuedAt, producer, collection, kind, linkId, itemId, summary}`, `linkId` the link id a creation was staged under, `itemId` the event an intent addresses, `summary` the summary of the body a creation pins or of the event an intent addresses, each `null` where it does not apply; then each parked row, any kind, as `{row, queuedAt, producer, collection, kind, attempts, unconfirmed, error}`. A kind SHALL print as the queue spells it, kebab-case (`add`, `calendar-reply`).

`queue show` SHALL print `{row, state, collection, kind, attempts, error, appliedAt, seq}`, `state` one of `pending`, `parked`, `unconfirmed`, `applied`, `unknown` and `reset`; a row anchored on a calendar of another account SHALL read `unknown`. `queue cancel` SHALL ask before cancelling unless `-y` is given, naming a row its performer sent as one that may have taken effect, and print `{row, kind, unconfirmed}`; it SHALL drop the row as the store's owner, taking that role for the call alone, refused `store-owned` while a sync holds it, and refuse a row applied already or unknown to the account's queue.

`pimdir status` stays the store's report, and `pimdir cancel` the `calendar-cancel` intent: neither is a queue command.

#### Scenario: A queued creation cancelled
- GIVEN a pimdir calendar `cal` where `event add` queued the event `Standup`
- WHEN `calendula --json pimdir queue list -k cal` runs, then `pimdir queue cancel -y <row>`
- THEN the first prints its row with `"collection":"cal"`, `"kind":"add"`, `"linkId"` the event's `UID` and `"summary":"Standup"`, the second `{"row":<row>,"kind":"add","unconfirmed":false}`, and the row leaves the queue

## MODIFIED Requirements

### Requirement: pimdir takes the reader and producer roles, the owner for one cancel
(backends) Renamed from "pimdir takes the reader and producer roles, never the owner". The pimdir backend SHALL read through a `PimdirReader` and write through a `PimdirProducer`, and SHALL NOT open a `PimdirStore`, but in `pimdir queue cancel`, which drops one queue row as Himalaya's does: it opens the store as its owner for that call alone, through `PimdirStore::open_current`, which creates nothing, and fails `store-owned` at once while a sync holds it.

### Requirement: Protocol-specific families
(commands) pimdir covers `queue list`, `queue show` and `queue cancel`, the store's queue as Himalaya reads and retracts it. `pimdir status` prints each parked action as `{row, calendarId, kind, error}`, `kind` kebab-case as the queue spells it, where the key was `action`, as cardamum's and Himalaya's rows say; its text column is `KIND`.

### Requirement: Text output is plain text
(commands) `pimdir queue list` SHALL print as Himalaya's and cardamum's do: `Account:` (the account name and `pimdir`), `Calendar:` (the calendar listed, `all` for every one) and, while it holds a row, `Cancel:` and the command cancelling one, above one line per row, its queue row, `queued`, `parked` or `unconfirmed`, when it was queued, its kind, its calendar, then its summary (the event an intent addresses when the store holds no summary) or why it was parked; `Nothing queued nor parked` when it holds none.

## REMOVED Requirements
