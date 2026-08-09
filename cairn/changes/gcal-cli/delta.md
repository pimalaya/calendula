---
cairn: change
change: gcal-cli
---

# Delta

## ADDED Requirements

### Requirement: gcal family
`gcal` SHALL expose what only the Calendar API has, gated behind the `gcal` cargo feature and building its own client from the account's `[gcal]` block, as every other protocol-specific family does.

It SHALL cover: `calendars`, the richer listing carrying the access role, the primary flag, the time zone and the default reminders a shared `Calendar` has no room for; `acl` with `list`, `create`, `update` and `delete` over the sharing rules; `free-busy`, the availability query over a window; `instances`, the occurrences a recurring event expands to; `move`, relocating an event to another calendar; `quick-add`, an event parsed server-side from a sentence; `colors`, the two palettes the API's colour ids refer to; and `settings`, the user's own.

### Requirement: An ACL rule is addressed by its scope
An ACL rule id is `<scope type>:<scope value>`, which the API mints. `create` and `update` SHALL take the scope and the role as flags and derive the id, so a caller never spells a composite id by hand, and `delete` SHALL accept the id a listing showed.

### Requirement: A window is given in days
`free-busy` and `instances` SHALL take `--from` / `--to` as inclusive days, as `event list` does, and map them onto the RFC 3339 bounds the API takes, so one date spelling serves the whole CLI.

### Requirement: What the gcal family declines
`channels` / `watch` SHALL NOT be exposed: a push channel delivers to an HTTPS endpoint the caller must host, which a CLI has not. `calendars.transferOwnership` SHALL NOT be exposed: it is an irreversible administrative act on a Workspace domain. `calendars.clear` SHALL NOT be exposed: it empties the primary calendar with no undo, and `calendar delete` covers the secondary case. These are declined on purpose, and SHALL be recorded as such rather than left as gaps.

## MODIFIED Requirements

### Requirement: Protocol-specific families
`caldav`, `gcal`, `pimdir` and `vdir` SHALL each expose what only that backend has, gated behind its own cargo feature. CalDAV covers `discover`, its own calendar listing (carrying the ctag, the sync token and the accepted component kinds), `create` and `delete`. gcal covers the half of the Calendar API iCalendar cannot express: sharing, availability, recurrence expansion, server-side parsing and the palettes. vdir covers its collection verbs including `rename`, which the shared API has no home for. pimdir covers `status`, reporting the source writes are attributed to, every source the store has been synced as, and how much of each calendar is downloaded.

A backend MAY exist without a family where its protocol adds nothing the shared surface lacks; a family SHALL NOT push protocol-specific concepts into the shared API instead.
