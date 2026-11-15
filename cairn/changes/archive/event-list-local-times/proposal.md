---
cairn: change
id: event-list-local-times
status: landed
created: 2026-10-19
---

# The event table shows times in the local zone

## Why

The START and END columns of `event list` print the property values as they are: `20261030T090000`, `20200521T150000Z`, `20230519`. calendula already resolves every time to an instant for `startsAt` and `endsAt` in the JSON, and the table is for a person to read (issue #17, point 1).

Without `--from`/`--to` the listing is the raw one, which on Google starts years back; sorting it by start would only sort the first page the backend returns, so the help points to a window instead (issue #17, point 2).

## What

- The table renders `startsAt`/`endsAt` in the local zone (`TZ`, then the system's), the one floating times are already read in: `2026-10-30 09:00` for a timed event, `2026-05-19` for a whole-day one.
- A whole-day event's end shows as its last day, not the exclusive end the JSON carries: a one-day event reads `2026-05-19` to `2026-05-19`.
- A time that did not resolve shows as iCalendar spells it, as the table did before.
- Squeezed into a narrow terminal or `--max-width`, the table truncates the id and the summary but keeps the start and end whole: `2026-10-1…` no longer says when.
- The `--json` payload is unchanged, `start` and `end` included.
- `event list --help` says that without a window the listing is the raw one, and points to `--from` or `event agenda` for what is coming.
