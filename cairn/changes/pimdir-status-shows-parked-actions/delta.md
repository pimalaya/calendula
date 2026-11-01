---
cairn: change
change: pimdir-status-shows-parked-actions
---

# Delta

## MODIFIED Requirements

### Requirement: Protocol-specific families
`configure` (alias `wizard`) SHALL run the account wizard.

`caldav`, `gcal`, `pimdir` and `vdir` SHALL each expose what only that backend has, gated behind its own cargo feature. CalDAV covers `discover`, its own calendar listing (carrying the ctag, the sync token and the accepted component kinds), `create` and `delete`.

gcal covers the half of the Calendar API iCalendar cannot express: sharing, availability, recurrence expansion, server-side parsing and the palettes. vdir covers its collection verbs including `rename`, which the shared API has no home for.

pimdir covers `status`, reporting the account being read, every account the store groups collections under, how much of each calendar is downloaded, how many creations are queued for the next sync, and every action the store's owner parked in those calendars, by its queue row (ROW), its calendar, its kind and the error that parked it; and `reply` and `cancel`, the scheduling intents only its sync engine can perform.

A backend MAY exist without a family where its protocol adds nothing the shared surface lacks; a family SHALL NOT push protocol-specific concepts into the shared API instead.
