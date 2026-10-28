---
cairn: tasks
change: gcal-send-updates
---

# Tasks

- [ ] `--send-updates` on `event create`, `event update` and `event delete`, refused by name off gcal before reaching the backend.
- [ ] Shared client: the choice reaches `create_item`, `update_item` and `delete_item` (see the open question).
- [ ] gcal: `update_item` and `delete_item` pass it as `sendUpdates`; `create_item` lets it replace the `all` of the insert path, passes `none` on the import path and refuses `all` / `external-only` there naming the `ORGANIZER`.
- [ ] Tests: absent flag keeps today's requests; each value reaches the request; the import-path refusal; the refusal off gcal on the three commands.
- [ ] Checked against Google with a second mailbox of mine: `update --send-updates none` and `delete --send-updates none` mail nothing; `create --attendee … --send-updates none` mails nothing and keeps the `UID`.
- [ ] `cargo test`, `cargo clippy --all-targets`, `cargo fmt`, build without default features.
- [ ] CHANGELOG: one `### Added` entry.
- [ ] Fold delta.md into cairn/spec/commands.md and cairn/spec/backends.md; append the log entry; mark `landed`.
