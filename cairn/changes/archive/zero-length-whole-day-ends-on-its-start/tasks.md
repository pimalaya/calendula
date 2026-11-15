---
cairn: tasks
change: zero-length-whole-day-ends-on-its-start
---

# Tasks

- [x] `table_times` in src/shared/event/list.rs: clamp a whole day's last day to its first.
- [x] Test: a zero-length whole-day event ends on its start (fails on the unclamped code).
- [x] `cargo test`, `cargo clippy --all-targets --all-features`, `cargo fmt --check`.
- [x] Fold delta.md into cairn/spec/commands.md; append the log entry; mark `landed`.
