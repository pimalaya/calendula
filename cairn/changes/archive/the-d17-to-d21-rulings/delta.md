---
cairn: delta
change: the-d17-to-d21-rulings
---

# Delta

## ADDED Requirements

### Requirement: A calendar created twice is a duplicate
(backends) A `calendar create` naming a calendar that already exists SHALL fail coded `calendar-duplicate`, writing nothing, wherever the backend can tell (pimalaya-cli error.md, *A code names the domain's noun*): vdir, a directory of that name in the home, ``Calendar `<id>` already exists``, and `vdir create` the same, ``Collection `<id>` already exists``; CalDAV, a MKCALENDAR answered with a body naming `DAV:resource-must-be-null` (RFC 4791 §5.3.1.1), or answered 405 or 412 when a listing then holds the calendar, the server's answer in `sources`; Graph, `ErrorFolderExists`, Exchange keeping one calendar per name. Google Calendar mints the id and lets two calendars share a name, so it holds no duplicate to refuse; pimdir serves no calendar create.

#### Scenario: A vdir calendar created twice
- GIVEN a vdir home holding the calendar `work`, named `Work`
- WHEN `calendula --json calendar create work Other` runs, then `calendula --json vdir create work -n Other`
- THEN each exits 1 coded `calendar-duplicate`, and `work` is still named `Work`

## MODIFIED Requirements

### Requirement: A server's refusal carries its code
(backends) The table gains `calendar-duplicate`: a CalDAV MKCALENDAR naming `DAV:resource-must-be-null`, or answered 405 or 412 over a calendar the listing holds; Graph's `ErrorFolderExists`; a vdir directory the home holds.

### Requirement: A JSON error carries a stable code
(commands) The table gains `calendar-duplicate`: a create names a calendar that already exists (backends, "A calendar created twice is a duplicate"); nothing was written.

### Requirement: msgraph family
(commands) `calendars list` SHALL print `{"calendars": [...], "nextPage"}` and `events list` `{"events": [...], "nextPage"}`, one Graph page, `nextPage` Graph's `@odata.nextLink` lifted out of the page, `null` on the last one, which `--page-token` takes back as is, the link carrying the calendar and the query, so `-k`, `--top`, `--select` and `--filter` are refused beside it; `events delta` one page of the calendar view delta over `--from` and `--to` (inclusive days, required without `--delta-link` or `--page-token`), `{"events": [...], "nextPage"}`, `nextPage` the `@odata.nextLink` draining the round, taken back by `--page-token`, the round's last page keeping Graph's raw `@odata.deltaLink`, which `--delta-link` takes back for the next round (pimalaya-cli output.md, *Paging resumes*). Their text SHALL end with a `Next page: <cursor>` line when one follows.

### Requirement: pimdir reads and retracts its queue
(backends) `queue list` SHALL print `{items, parked}`: each creation and each intent waiting, the rows no listing shows (an update or a removal addresses an item a listing shows), then each parked row, any kind, every row in the family's one shape (pimalaya-cli output.md, *A queue row keeps one shape*): `{row, kind, collection, linkId, itemId, producer, queuedAt, attempts, unconfirmed, error, summary}`, every key printed. `kind` SHALL be the action as the store holds it, never re-spelled (`add`, `update`, `calendar-reply`); `linkId` the link id of the item a row creates; `itemId` the stored event a row addresses; `attempts` the sync engine's tries; `unconfirmed` whether a parked row's outcome is unknown; `error` `null` while the row is pending, the store's text once it is parked; `summary` the summary of the body a row pins, else of the event it addresses; each `null` where it does not apply.

## REMOVED Requirements
