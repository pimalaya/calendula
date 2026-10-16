---
cairn: tasks
change: gcal-anchors-floating-times
---

# Tasks

- [x] `EventFieldsArgs::anchor_floating`, setting `--time-zone` to the local zone on gcal and msgraph when a time flag is given and no zone flag.
- [x] Call it from `event create` and `event update`, next to `check_backend`.
- [x] Tests: both backends anchor, the others keep the time floating; `--time-zone`, a date and a `Z` time win; no time flag, no zone; no IANA name, nothing anchored.
- [x] Checked that the local zone resolves to its IANA name, from the system and from `TZ`.
- [x] `cargo test`, `cargo clippy --all-targets`, `cargo fmt`.
- [x] CHANGELOG: one `### Changed` entry; the `--time-zone` help says so.
- [x] Fold delta.md into cairn/spec/commands.md; append the log entry; mark `landed`.
