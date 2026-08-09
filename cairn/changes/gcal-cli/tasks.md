---
cairn: tasks
change: gcal-cli
---

# Tasks

- [x] `gcal/client.rs`: a `GcalClient` bundling the connected client and the merged account, plus `build_gcal_client`, as the CalDAV family has.
- [x] `gcal/cli.rs`: the subcommand tree, wired into `CalendulaCommand` behind the `gcal` feature.
- [x] `gcal/calendars.rs`: the richer listing (access role, primary, time zone, colours, default reminders).
- [x] `gcal/acl.rs` and `gcal/acl/`: `list`, `create`, `update`, `delete` over the sharing rules.
- [x] `gcal/free_busy.rs`: the availability query over a window and a set of calendars.
- [x] `gcal/instances.rs`, `gcal/quick_add.rs`, `gcal/move_event.rs`: the three event operations the shared API cannot carry.
- [x] `gcal/colors.rs` and `gcal/settings.rs`: the palettes and the user settings.
- [x] Tests: the scope round-trips through its wire spelling, the window maps onto the RFC 3339 bounds, and a rule id is derived from its scope.
- [x] `main.rs`, `config.sample.toml` and the README.
- [x] Fold the delta into `cairn/spec/commands.md`; write `cairn/log/2026-08-09-gcal-cli.md`.
