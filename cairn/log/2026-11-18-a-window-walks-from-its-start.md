---
cairn: log
change: a-window-walks-from-its-start
landed: 2026-11-18
---

# A window walks from its start

Landed without a proposal: CAL-12 and CAL-8 of the deep analysis.

- **The walk starts at the window** (src/shared/event/expand.rs): `Context::walk` takes a `floor`, the window's start less a day either way (`Window::floor`), and skips the recurrence set to it, less the event's length, through ical-rs `IcalRecurSetExpand::skip_to`; the overrides the skip passes are still placed by the loop after the walk. `occurrence` skips straight to the identity it looks for. A minutely series begun 2026-01-01 listed nothing on 2026-06-01 once the walk passed `MAX_STEPS`; it lists its 1,440 occurrences (`a_series_begun_long_before_the_window_lists_in_it`).
- **CAL-8** goes with it: the analysis's daily series from 2020 in a `Romance Standard Time` the calendar defines listed one day in 24.9 s on the fac7873 debug binary and in 0.06 s on this one (one run each, `TZ=Europe/Paris event list --from 2026-11-16 --to 2026-11-16`, repro/verify/tz of the analysis). The per-occurrence zone resolution the analysis named is untouched; it no longer runs over years of instances.
- **CAL-12, paging**: a window given `-s` read one page of items in backend order and sorted inside it, the page line counting items that listed nothing. It now reads every item of the window, sorts the occurrences, and pages them (`page_window`), the page line counting what it shows. Reproduced at fac7873 on a vdir calendar whose earliest event sorts last by file name: page 1 printed `Standup`, `Review` and left `Earliest` for page 2; it prints `Earliest`, `Standup` now (`a_window_pages_its_events_in_start_order`, tests/event_list.rs).

Capabilities moved: commands.
