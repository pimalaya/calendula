---
cairn: log
change: event-build-projects
date: 2026-10-06
---

# `event build` projects what it built

A loose iCalendar, such as an invitation attached to a mail, is now read the way a calendar's events are, with no account and no backend: MOA drops its own invitation parser for it.

## What landed

- `event build --json` answers `{contents, method, events}` (`EventBuildOutput`, registered as `calendula-event-build`); the other families keep `{contents}`, and the text output is unchanged.
- The event projection gains `sequence` (0 when absent) and `zoneAssumed` (a `TZID` neither the database nor the calendar defines, read in the local zone, which `TZ` sets).
- Unit tests `a_zone_nothing_defines_is_assumed_and_one_defined_is_not`, `the_sequence_reads_as_a_number_and_defaults_to_zero`, `the_method_is_the_calendar_one_uppercased`; the JSON shape test carries both keys.
- Capability moved: `commands` (the event projection; an event build answers its projection).

## Verification

`cargo test --all-features` and `cargo clippy --all-features --all-targets` clean; `TZ=Europe/Paris calendula --json event build -` on an Outlook invitation (Windows `TZID` with its `VTIMEZONE`, `DURATION`) gives `2026-10-19T09:00:00+02:00` to `10:00:00+02:00`, `method` `REQUEST`, `sequence` 2.
