---
cairn: change
id: the-x10-output-shapes
status: landed
created: 2026-11-15
---

# The X10 output shapes

> Task X10 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15/tasks.md): N0 (live suites off push) and the schema review's calendula findings A3, A11, A12, A14 and B13 (reports/schema-review-status-2026-11-15.md there), before the 0.3 major. pimalaya-cli's cairn/spec/output.md is the contract.

## Why

- N0: every push to master ran the credentialed live suites, burning Google's calendar quota, hitting Graph throttling and colliding with the owner's live testing.
- A3: `pimdir reply` and `pimdir cancel` printed the queue row as `queued`, an integer under the key every receipt uses for a boolean, and the event as `eventId`, where every other queue output says `row` and every event output `id`.
- A11: an event printed each time twice, `start` and `end` iCalendar-spelled (`20261112T090000Z`) beside `startsAt` and `endsAt` in RFC 3339. Values have one spelling (output.md, *Values have one spelling*).
- A12: `uid` and `summary` printed `""` for an event carrying none (output.md, *Absent is null*).
- A14: the agenda JSON keyed floating local times (`2026-11-12T10:00:00`, no offset) to bare labels, with no item id to open what it shows.
- B13: `gcal free-busy` printed Google's per-calendar errors flattened to strings under `errors`, the key whose type across calendula is the item error `{id, code, error, sources}`.

## What

- N0: .github/workflows/tests.yml runs the `live` job on `workflow_dispatch` and weekly (`0 4 * * 1`, calendula's slot), never on push; the offline jobs are unchanged. CONTRIBUTING.md says so. The live tests create no calendar (they only read), so the long-lived calendar rule asks nothing here.
- A3: `{row, id, calendarId, recurrenceId, partstat, comment, source}`.
- A11: one pair, `start` and `end`, RFC 3339 with the offset the time was written at (`Z` for UTC), or `YYYY-MM-DD` for a whole day with an exclusive end, `null` when the time does not resolve, beside `allDay` and `timeZone`. The names are the ones the text columns and the iCalendar properties (`DTSTART`, `DTEND`) already use, and the ones `gcal free-busy` and the busy periods print; the `startsAt` pair is dropped since it never shipped in a release, so dropping it breaks only development builds, while `start` and `end` were in 0.2. The value form follows output.md: a timestamp read from content keeps its offset, a date is `YYYY-MM-DD`. The iCalendar spelling stays in the item's `contents`.
- A12: `uid` and `summary` are `null` when the event carries none, as `description` and `location` are, and as cardamum's card row prints `uid` and `fullName`.
- A14: `{events: [{id, start, label}], errors}`, the events in the order the text prints them, `start` the event row's value.
- B13: the rows print `queryErrors`, Google's `{domain, reason}` objects raw; `errors` leaves the output.

Breaking against 0.2: A11 (`start` and `end` change form), A12 (`summary` was `""`), A14 (the agenda root), B13 (free-busy `errors`). `pimdir reply` and `pimdir cancel` are new since 0.2: A3 rewords their [Unreleased] line.
