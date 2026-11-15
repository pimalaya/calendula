---
cairn: tasks
change: gcal-delete-if-match
---

# Tasks

- [x] `delete_item`: pass `if_match` to io-gcal's `event_delete` instead of refusing it.
- [x] Checked against Google: a delete naming a stale etag fails with 412 and leaves the event; naming the current one deletes it (Google keeps it readable as CANCELLED, and it no longer lists).
- [x] `cargo test`, `cargo clippy --all-targets`, `cargo fmt`, build without default features.
- [x] CHANGELOG: amend the unreleased `--if-match` entry.
- [x] Fold delta.md into cairn/spec/backends.md and cairn/spec/commands.md; append the log entry; mark `landed`.
