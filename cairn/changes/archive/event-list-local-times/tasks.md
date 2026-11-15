---
cairn: tasks
change: event-list-local-times
---

# Tasks

- [x] `table_times` in `src/shared/event/list.rs`: a timed event at its instants in the local zone, a whole-day one by its days with an inclusive end, the iCalendar spelling as a fallback.
- [x] START and END keep their content width when the table is squeezed (`ColumnConstraint::ContentWidth`, as himalaya keeps its id).
- [x] `event list --help`: without a window the listing is the raw one; `--from` or `event agenda` for what is coming.
- [x] Tests: a timed event in Europe/Madrid, a two-day and a one-day whole-day event, unresolved times.
- [x] Checked against a Google calendar: timed and whole-day events over a window.
- [x] `cargo test`, `cargo clippy --all-targets --all-features`, `cargo fmt --check`.
- [x] CHANGELOG: one `### Changed` entry.
- [x] Fold delta.md into cairn/spec/commands.md; append the log entry; mark `landed`.
