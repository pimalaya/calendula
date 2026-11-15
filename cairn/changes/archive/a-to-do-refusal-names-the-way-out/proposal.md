---
cairn: change
id: a-to-do-refusal-names-the-way-out
status: landed
created: 2026-10-30
---

# A to-do refused by an events-only calendar names the way out

## Why

io-pimdir 1df2b70 and 8907303, which calendula pinned on 2026-10-29 (commits f3a7330 and 84aa00c), gate a VTODO landing in a calendar collection whose source states `x-vtodo` `none` there or source-wide (pimdir STORAGE §15.6): `todo create` and `todo update`, or `item create` and `item update` with a VTODO body, are refused before anything is queued. That behaviour change shipped with the pin bumps alone, with no CHANGELOG line, no spec requirement and no log entry, against the repo's forcing rule.

The refusal also read as the store's generic one, `Stage the pimdir action: Source caldav does not support x-vtodo: events only` (src/pimdir/backend.rs:833, reproduced 2026-10-30), naming a capability a user does not know and no way out.

## What

- The refusal names the cause and the way out: the calendar takes events only (the source's detail when it gives one), so store the to-do in another calendar with `-k/--calendar`.
- Spec requirement for the refusal, CHANGELOG Changed entry for the behaviour the pin bumps brought, log entry.

## Out

- Offering only calendars that take to-dos (`PimdirReader::accepts_vtodo`) in a listing: the to-do family leaves calendula with change events-only.
