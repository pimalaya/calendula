---
cairn: tasks
change: event-field-flags
---

# Tasks

- [ ] Settle the open questions of the proposal.
- [ ] `EventFieldsArgs` (`src/shared/event/fields.rs`), one shared argument, as cardamum's `CardFieldsArgs`; `apply` on the CST, byte-faithful for every line no flag names.
- [ ] Zones: `--time-zone` writes the `TZID` and mints the `VTIMEZONE` (ical-rs `tzdb::vtimezone`), once per zone, before the VEVENT.
- [ ] `event build`, `event create`, `event update`: source, flags, `-i`, in that order; the "nothing to build" error names the flags.
- [ ] Check a build or create from flags alone; refuse flags on a source holding several VEVENTs.
- [ ] Help: each flag documented, the composer named as the complete surface.
- [ ] Tests: each flag, the replace rule, a byte-for-byte untouched source, zones, the refusals; `event build` JSON with flags.
- [ ] Fold into `spec/commands.md`, log, changelog.
