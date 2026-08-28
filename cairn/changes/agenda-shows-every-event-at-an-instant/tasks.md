---
cairn: tasks
change: agenda-shows-every-event-at-an-instant
---

# Tasks

- [x] Decide the payload shape for `event agenda --json`: a list of labels per instant. It is the least breaking of the two, the top-level object and its keys being untouched, where a flat list of (instant, label) pairs would change the payload's own type. The flag is `--json`, not `--output json`, which the proposal misnamed.
- [x] Hold every event at an instant in the agenda's own state, dropping the one-label-per-key map.
- [x] State the order between two events at one instant (label, then item id, so it is total and stable) and implement it there.
- [x] Text output renders every event at the instant without repeating the time column, so a duplicated pair reads as two lines under one time rather than as two identical rows.
- [x] Tests: two events at one instant both render, in the stated order, in the text and JSON outputs; a single event's rendering is unchanged.
- [x] `cargo test`, `cargo clippy --all-targets`, `cargo fmt`.
- [x] CHANGELOG: one `### Added` entry for `event agenda`, stating the final shape. `event agenda` landed after v0.1.0 and has never shipped, so a `### Changed` (breaking) entry would announce a break against a payload no release ever emitted, and a `### Fixed` one a fix to a version nobody ran. `[Unreleased]` is the net diff against the last release, so the two collapse into the command's own entry.
- [x] Fold `delta.md` into `cairn/spec/commands.md`; append the log entry; mark `landed`.
