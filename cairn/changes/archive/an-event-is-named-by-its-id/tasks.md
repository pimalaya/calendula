---
cairn: tasks
change: an-event-is-named-by-its-id
---

# Tasks

- [x] src/shared/event/delete.rs: `resolve` and `pick` go, `check` reads the id and answers a miss `item-not-found` naming `event search <UID>`.
- [x] Tests failing on the code before: tests/event_list.rs `a_uid_is_no_event_id` (the old binary deleted `a.ics`); src/shared/event/delete.rs `a_missing_id_is_not_found_and_lists_nothing` (two requests before, one now). tests/google.rs's ignored `gcal_event_delete_of_an_unknown_uid_is_not_found` keeps its expectation, its comment updated.
- [x] MCP instructions say the event tools take no UID.
- [x] CHANGELOG, MIGRATION; fold the delta; log.
