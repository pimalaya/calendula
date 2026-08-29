---
cairn: tasks
change: agenda-shows-every-event-at-an-instant
---

# Tasks

- [x] Decide the payload shape for `event agenda --json`: a list of labels per instant. It is the least breaking, the top-level object and its keys staying untouched where a flat list of (instant, label) pairs would change the payload's type. The flag is `--json`, not the `--output json` the proposal named.
- [x] Hold every event at an instant in the agenda's own state, dropping the one-label-per-key map.
- [x] State the order between two events at one instant (label, then item id, so it is total and stable) and implement it there.
- [x] Text output renders every event at the instant without repeating the time column, so a duplicated pair reads as two lines under one time rather than as two identical rows.
- [x] Tests: two events at one instant both render, in the stated order, in the text and JSON outputs; a single event's rendering is unchanged.
- [x] `cargo test`, `cargo clippy --all-targets`, `cargo fmt`.
- [x] CHANGELOG: one `### Added` entry for `event agenda`, stating the final shape. It landed after v0.1.0 and never shipped, so a breaking `### Changed` would announce a break no release emitted and `### Fixed` a fix to a version nobody ran; `[Unreleased]` being the net diff, both collapse there.
- [x] Fold delta.md into cairn/spec/commands.md; append the log entry; mark `landed`.
