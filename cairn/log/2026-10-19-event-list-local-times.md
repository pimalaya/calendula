---
cairn: log
change: event-list-local-times
landed: 2026-10-19
---

# The event table shows times in the local zone

`EventListOutput` renders the START and END columns through `table_times` in `src/shared/event/list.rs`: a timed event at its `start_secs`/`end_secs` in the local zone (`jiff` `TimeZone::try_system`, `TZ` then the system's), spelled `YYYY-MM-DD HH:MM`; a whole-day event by its `startsAt` and the day before its exclusive `endsAt`; the iCalendar spelling when a time did not resolve. Both columns keep their content width, so a narrow table truncates the id and the summary instead. The JSON payload is untouched. `event list --help` now says that without a window the listing is the raw one and points to `--from` or `event agenda`.

Verification: `cargo test`, `cargo clippy --all-targets --all-features` and `cargo fmt --check`; `event list --from --to` against a Google calendar.

Capabilities moved: **commands** (the event table shows local times; the unwindowed listing stays raw, modified).
