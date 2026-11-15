---
cairn: change
id: series-exceptions-follow-the-listed-range
status: landed
created: 2026-11-15
---

# Series exceptions follow the listed range

## Why

[calendula#31](https://github.com/pimalaya/calendula/issues/31): on Microsoft 365, `event list` fails whole as soon as the calendar holds a recurring series with no end date.

```
Error: Microsoft Graph API returned HTTP 400 (ErrorInvalidRequest): [...] The range between the start and end dates is greater than the allowed range. Maximum number of years: 5
```

`exceptions` (src/msgraph/backend.rs) reads a series master's instances over `series_window`: the series' own range, an open-ended one ending `OPEN_SERIES_YEARS` (5) after its start, then one day added for the exclusive end. That is five years and a day, which Graph refuses; a bounded series longer than five years is not capped at all. The error propagates and fails the listing.

The reporter's fix caps the window at `start + 5 years`. That ends the 400 but keeps a second fault: the window is anchored at the series' start, not at the dates listed. A weekly meeting started in 2019 with no end is read for 2019 to 2024 only, so this week's cancelled or moved occurrences are never read, and the listing shows them as the plain series would.

neverest met the same 400 on 2026-10-04 and solved it in its own code; io-msgraph's change instances-read-in-windows-graph-accepts lifts the window cutting into the Graph client, which this change builds on.

## What

- **The range read.** `exceptions` takes the range its caller lists:
  - `event list`, `item list` and the other listings with `--from`/`--to`: that range;
  - a listing without one, `item get` and `item update`: today ± five years, as neverest reads an open-ended series.
- **The windows.** That range is cut by io-msgraph's `MsgraphPatternedRecurrence::instance_windows`, so no request exceeds Graph's five years; `series_window` and `OPEN_SERIES_YEARS` go. Each window's pages are walked as today.
- **A failed window.** A refusal of one window keeps today's rule: the event becomes an item error coded by the refusal, the other events listed (cairn/spec/backends.md, Graph entry). It no longer fails the listing whole.
- **An update.** `item update` reads its base over the same range as it reads the current event, so an exception outside it is in neither and left untouched by the delta.

## Scope / non-goals

- An exception outside the range read is not in the item's contents: the iCalendar of a series is complete within the range only. `item get` says nothing of it today either.
- Needs io-msgraph 0.9.1. Until then, the reporter's cap may land on its own as the fix of the 400, this change following.
