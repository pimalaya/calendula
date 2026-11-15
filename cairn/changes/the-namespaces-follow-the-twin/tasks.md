---
cairn: tasks
change: the-namespaces-follow-the-twin
---

# Tasks

- [x] Proposal and delta, the command map with cardamum's counterpart and the deviations.
- [x] `caldav`: src/caldav/cli.rs (the method list, `list`, `ls` and `create` hidden aliases), discover.rs (`server`, unresolved as `null`), propfind.rs (from list.rs), proppatch.rs, mkcalendar.rs (from create.rs), get.rs, put.rs, delete.rs (an item or a calendar), report/ (`query`, `multiget`, `sync`, `raw`, entries.rs); `unreadable_row` shared from src/caldav/backend.rs.
- [x] `msgraph`: src/msgraph/client.rs (`build_msgraph_client`, `connect` shared with the backend, `event_refusal`, `calendar_refusal`), cli.rs, calendars/, events/, profile/, request.rs; src/shared/raw_json.rs; `IdReceipt` built with msgraph.
- [x] `vdir item`: src/vdir/item/ from cardamum's, the kind sniffed as iCalendar unless the bytes open a vCard, `get` printing `encoding`.
- [x] `gcal events`: src/gcal/events.rs and events/ (instances.rs, move_event.rs, quick_add.rs moved), the flat names hidden commands in src/gcal/cli.rs.
- [x] The command tree (src/cli.rs), the schema registry (src/json_schema.rs), src/main.rs header; src/script.rs, the scripted server of cardamum, for the tests.
- [x] Tests: src/cli.rs `namespace_tests` (every new path parses, the former names reach the same command, hidden), src/caldav/propfind.rs (the home listing keeps the `caldav list` shape key for key, a calendar enumerates its items), src/caldav/report/ (the filter the flags build, the errors of a query and a multiget, the raw path and its refusals), src/msgraph/client.rs (a missing event or calendar coded), src/msgraph/events/render.rs, src/vdir/item/get.rs (text or base64).
- [x] CHANGELOG (Added, Changed), MIGRATION.md (Commands and flags, the names in Text output and JSON output), README (features, the quick-add line).
- [x] Fold the delta into [commands](../../spec/commands.md) and [backends](../../spec/backends.md); log.
