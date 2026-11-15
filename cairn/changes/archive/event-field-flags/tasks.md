---
cairn: tasks
change: event-field-flags
---

# Tasks

- [x] Settle the open questions of the proposal (2026-10-06).
- [x] `EventFieldsArgs` (`src/shared/event/fields.rs`), one shared argument, as cardamum's `CardFieldsArgs`; `apply` on the CST, byte-faithful for every line no flag names.
- [x] Zones: `--time-zone` writes the `TZID` and mints the `VTIMEZONE` (ical-rs `tzdb::vtimezone`), once per zone, before the VEVENT.
- [x] `event update --start` alone moves `DTEND` (or keeps `DURATION`), the length kept.
- [x] `event build`, `event create`, `event update`: source, flags, `-i`, in that order; the "nothing to build" error names the flags.
- [x] Check a build or create from flags alone; refuse flags on a source holding several VEVENTs.
- [x] Help: each flag documented, the composer named as the complete surface.
- [x] Tests: each flag, the replace rule, a byte-for-byte untouched source, zones, the refusals; `event build` JSON with flags.
- [x] Fold into `spec/commands.md`, log, changelog.
