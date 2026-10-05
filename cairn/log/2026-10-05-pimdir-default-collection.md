---
cairn: log
change: pimdir-default-collection
date: 2026-10-05
---

# The default calendar is marked

pimdir draft-04 adds `collections.role`, which neverest fills from what each server states, `default` marking the calendar a new item goes to when none is named (Google's primary calendar, Graph's default calendar and Contacts folder, People's one book, CalDAV's `schedule-default-calendar-URL`). io-pimdir `ef8eae0` reads it.

## What landed

- Cargo.toml, Cargo.lock: io-pimdir by git rev `ef8eae0`.
- `Calendar` gains `default: bool`, set on the pimdir backend from the role, `false` elsewhere.
- Unit test `the_default_calendar_is_the_one_its_store_marks`.

## Verification

`cargo test --all-features` and `cargo clippy --all-features --all-targets` clean.
