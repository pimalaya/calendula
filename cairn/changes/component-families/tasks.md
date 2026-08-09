---
cairn: tasks
change: component-families
---

# Tasks

- [x] `shared/todos.rs`: the `Todo` projection (summary, due, status, priority, percent complete) over a `CalendarItem`.
- [x] `shared/journals.rs`: the `Journal` projection (summary, start).
- [x] `shared/todos/` and `shared/journals/`: `cli`, `list`, `read`, `create`, `update`, `delete`, mirroring the `event` family.
- [x] `config.rs` and `account/context.rs`: `todo.list` and `journal.list` page sizes and column colours, merged like the others.
- [x] `cli.rs` and `main.rs`: the two families in the shared group, and the header's `event against item` section rewritten around three component views.
- [x] Tests: each projection keeps its own kind and drops the others, a malformed item projects nothing, and the todo columns read the properties they claim.
- [x] `config.sample.toml` and the README.
- [x] Fold the delta into `cairn/spec/commands.md` and `cairn/spec/config.md`; write `cairn/log/2026-08-09-component-families.md`.
