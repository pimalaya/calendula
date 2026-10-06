---
cairn: log
change: event-flags-for-clients
date: 2026-10-06
---

# Field flags a client can edit events with

A client writing events through the field flags (MOA) no longer needs iCalendar of its own to empty a field, revise an event or ask for an online meeting.

## What landed

- An empty value to `--summary`, `--description`, `--location`, `--url`, `--organizer` removes the property; a repeated `--categories` or `--attendee` drops its empty values, so `--categories ""` or `--attendee ""` alone removes every instance. Removing an absent property leaves the event byte for byte. Times, `--duration`, `--status`, `--transparency` and `--sequence` still refuse an empty value. `replace` in `src/shared/event/fields.rs` now keys on property names (`IcalPropName`), so an `X-` property is written the same way.
- `event update` revises every VEVENT it writes (`fields::revise`): `DTSTAMP` to the time of the write in UTC, `LAST-MODIFIED` to the same instant when present, a missing `DTSTAMP` added after the last property, every other line kept. It runs in the shared update pipeline (`EventUpdateCommand::execute_at`, the clock injected), after the flags or once the composer hands the edit back, so every backend gets it; gcal consumes `DTSTAMP` and `LAST-MODIFIED` as its projection already said.
- `--online-meeting` writes `X-PIMDIR-ONLINE-MEETING:TRUE` (pimdir STORAGE Annex B.1), replacing any instance. `event create` and `event update` refuse it by name off pimdir (`EventFieldsArgs::check_backend`, `CalendarClient::backend_name`). The pimdir backend refuses a body newly asking for one, before the blob is staged, unless a source syncing the calendar declares `calendar.online-meeting` fully or in part: io-pimdir's gate already refused a declared source lacking it, and calendula now refuses a store declaring nothing too. The wording goes through the same staging error as the scheduling refusal (`staging_error`).
- Capabilities moved: `commands` (empty flags, update revision, `--online-meeting`), `backends` (the pimdir online-meeting refusal).

## Verification

`cargo test --all-features` (174 tests, 22 new: 10 in `fields.rs`, 2 in `event/build.rs`, 5 in `event/update.rs` over vdir and pimdir with an injected clock, 1 in `event/create.rs`, 4 in `pimdir/backend.rs`), `cargo clippy --all-features --all-targets` clean, `cargo fmt`, `cargo build --no-default-features --features pimdir`, plus the vdir-only and caldav-only sets.
