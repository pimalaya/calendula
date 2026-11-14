---
cairn: tasks
change: an-unknown-key-is-refused
---

# Tasks

- [x] `deny_unknown_fields` on `Config` and `AccountConfig`; no `flatten` in the config.
- [x] An `IgnoredAny` field for each backend left out of the build.
- [x] `calendar.list`, `event.list.table` and `item.list.table` refused by name.
- [x] Tests: an unknown key at the root and in an account, the colour tables by name (each failing on the old code), a backend and nested tables (already refused), every backend block loading in every build, the sample loading in every variant in every build.
- [x] CHANGELOG breaking entry and Removed entry, MIGRATION.md, the src/main.rs header.
- [x] Fold the delta into config.md, write the log entry, set `landed`.
