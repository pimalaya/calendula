---
cairn: change
change: the-contract-audit-rulings
---

# Delta

## ADDED Requirements

### Requirement: TLS is set per account and per backend
Every network backend block SHALL take a `tls` table spelled as pimalaya-config's toml.md writes it (`provider`, `rustls.crypto`, `cert`, `client.cert`, `client.key`), and an account MAY carry one its backends inherit field by field, `client` as one (config.md).

### Requirement: An event search runs the pimdir query language
`event search <QUERY>...` SHALL take the pimdir SEARCH query language, answered by a pimdir store's index and refused `capability-missing` (`event.search`) on every other backend, printing `{events, nextPage, complete, errors}` (commands.md).

### Requirement: An event is read by its UID
`event read --uid <UID>` SHALL read the item carrying a VEVENT of that `UID`, in place of an id; none fails `item-not-found` (replaces "An event is found by its UID").

## MODIFIED Requirements

### Requirement: Capability registry
A capability SHALL be named after the unified command it stands for, `<noun>.<verb>`, or `<noun>.<verb>.<argument>` for a flag of that command spelled as its long flag; `event.search` joins the registry.

### Requirement: caldav speaks the protocol
`caldav delete <CALENDAR> <ITEM-ID>` SHALL delete one item, its id required; `caldav rmcol <CALENDAR>` SHALL delete the calendar.

### Requirement: The event projection
An event's JSON SHALL carry `calendarId` after `id`, both `null` for an event no item holds (`event build`).

### Requirement: pimdir reads and retracts its queue
`queue list` SHALL print `{actions, parked, errors}`; `queue show` the list row plus `state`, `appliedAt` and `refusals`, an unknown row `row-not-found`; `queue cancel` codes `row-not-found` and `confirmation-required`; `pimdir status` parked rows name `collection` (backends.md).

### Requirement: account check fails when a backend fails
`account check` SHALL print pimalaya-cli's list of accounts, each backend row carrying `tls`.

### Requirement: A JSON error carries a stable code
`row-not-found`, `confirmation-required`, `page-token-invalid` and `sending-closed` join the vocabulary; `stdin-taken` covers the composer in a session, `item-invalid` a source that is no iCalendar, `calendar-duplicate` a pimdir calendar the store holds, before `capability-missing`.

### Requirement: A protocol refusal is a result
Every protocol command SHALL end its `--help` with the meaning of exit status 3.

### Requirement: A page names the next one
A token page SHALL print `Next page: --page-token <token>`.

### Requirement: A bare invocation shows the help
A bare `calendula --json` SHALL fail coded `usage-invalid`, exit 2.

### Requirement: Text output is plain text
`vdir list` SHALL print records, and `pimdir status` the calendars as ID, ITEMS, DOWNLOADED, QUEUED, NAME, as cardamum's.

### Requirement: A write reaches attendees under --allow-send alone
The refusal SHALL be coded `sending-closed` (mcp.md).

## REMOVED Requirements

### Requirement: An event is found by its UID
Replaced by "An event is read by its UID"; `event find` is no more an alias.
