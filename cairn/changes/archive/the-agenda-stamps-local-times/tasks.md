---
cairn: tasks
change: the-agenda-stamps-local-times
---

# Tasks

- [x] Tests: tests/event_list.rs `the_agenda_stamps_an_event_in_the_local_zone` (a `Z` event at 09:00 keyed `10:00` under `TZ=Europe/Paris`, printed `04:00` under `America/New_York`), failing on the code before; src/shared/event.rs `a_timed_start_reads_on_the_local_clock`.
- [x] `Event::start_at` (src/shared/event.rs), `CalControl::zone` (src/shared/event/agenda.rs).
- [x] CHANGELOG (Fixed); MIGRATION.md (the agenda keys).
- [x] Fold the delta into [commands](../../../spec/commands.md); log.
