---
cairn: tasks
change: event-occurrences-and-scheduling-intents
---

# Tasks

- [x] Expansion and zone resolution (src/shared/event/expand.rs), jiff and ical-rs `tzdb` always on
- [x] The richer `Event` projection and its JSON
- [x] `event list` expands a window; `event read -r`; `event find`
- [x] Local range filters keep a recurring item by its occurrences; pimdir reads `recurring` and `until` off an undownloaded summary
- [x] `pimdir reply` and `pimdir cancel` queue the Annex B.2 intents with their performer
- [x] Tests: daily COUNT, weekly BYDAY with UNTIL and EXDATE, an override, an override moved from past the window, an all-day series, the Paris autumn change, a VTIMEZONE-only zone, the JSON shape, the queue rows
- [x] Feature matrix, tests, clippy, fmt
- [x] CHANGELOG, spec fold, log
