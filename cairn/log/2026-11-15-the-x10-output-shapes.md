---
cairn: log
change: the-x10-output-shapes
landed: 2026-11-15
---

# The X10 output shapes

Task X10 of the CLI alignment round: N0 and the schema review's calendula findings A3, A11, A12, A14 and B13. Capabilities moved: commands, backends, packaging.

- **N0**: .github/workflows/tests.yml runs the `live` job by hand and weekly (`0 4 * * 1`), never on push; CONTRIBUTING.md says so. The live tests only read, so no calendar moves onto a long-lived one; packaging states the rule for a future test that needs one.
- **A3**: `pimdir reply` and `pimdir cancel` print `{row, id, calendarId, recurrenceId, partstat, comment, source}`, where they printed `queued` and `eventId` (src/pimdir/intent.rs).
- **A11**: an event prints one time pair, `start` and `end`, RFC 3339 with the written offset or a date, `null` when unresolved; `startsAt` and `endsAt` are gone and the iCalendar spelling is kept unprinted for windowing and the text fallback (src/shared/event.rs, expand.rs, list.rs; the pimdir search and index).
- **A12**: `uid` and `summary` are `null` when absent, as cardamum's card row.
- **A14**: `event agenda --json` prints `{events: [{id, start, label}], errors}` (src/shared/event/agenda.rs).
- **B13**: `gcal free-busy` prints the API's error objects raw under `queryErrors` (src/gcal/free_busy.rs).
- **Breaking against 0.2**: A11, A12, A14, B13, each with a CHANGELOG line and a MIGRATION.md entry; A3 rewords the [Unreleased] line of commands new since 0.2.
