---
cairn: tasks
change: a-calendar-update-clears-and-colours-first
---

# Tasks

- [x] Tests, src/caldav/backend.rs over a scripted server recording its requests: `a_cleared_description_is_removed` (one PROPPATCH holding `<D:remove>` and the new name), `an_ignored_colour_writes_nothing_else` (one PROPPATCH, the colour alone, the update failing), `a_colour_dropped_on_create_is_read_back` (a PROPFIND after the MKCALENDAR); src/gcal/backend.rs `a_cleared_description_is_sent_empty`. All four failed on the code before (no remove, one PROPPATCH holding all three, no read back, no `description` field).
- [x] `proppatch` and `accepted` in src/caldav/client.rs; `update_calendar`, `patch` and the colour read back in src/caldav/backend.rs.
- [x] gcal `update_calendar` and `calendar_from` (src/gcal/backend.rs).
- [x] CHANGELOG (Fixed, Changed).
- [x] Fold the delta into [backends](../../spec/backends.md) and [commands](../../spec/commands.md); log.
