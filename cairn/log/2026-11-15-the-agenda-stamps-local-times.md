---
cairn: log
change: the-agenda-stamps-local-times
landed: 2026-11-15
---

# The agenda stamps local times

`event agenda` stamps and files each event at its start in the local zone `event list` reads, where it stamped a `Z` event at its UTC wall time whatever `TZ` was, text and JSON keys alike. Line K23 of the CLI alignment round, from the live testing campaign (C-F1). Capability **commands** moved.

- **src/shared/event.rs**: `Event::start_at(zone)`, a timed event at its instant on `zone`'s clock.
- **src/shared/event/agenda.rs**: `CalControl::zone`, the local zone.
- **Evidence**: tests/event_list.rs `the_agenda_stamps_an_event_in_the_local_zone` failed on the code before (`2026-11-12T09:00:00` under `TZ=Europe/Paris`).

CHANGELOG (Fixed), MIGRATION.md. Spec: commands ("An agenda expands recurring series").
