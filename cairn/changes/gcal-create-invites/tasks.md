---
cairn: tasks
change: gcal-create-invites
---

# Tasks

- [ ] `create_item`: an event with an `ATTENDEE` and no `ORGANIZER` goes through `events.insert` with `GcalSendUpdates::All`, its `iCalUID` in the body; the rest as today.
- [ ] Tests: the insert path and its `sendUpdates=all`, and that an event with an `ORGANIZER`, or with no attendee, keeps its current path.
- [ ] Checked against Google with a second mailbox of mine: `event create --attendee` mails the invitation and the event keeps the `UID` it was built with.
- [ ] `cargo test`, `cargo clippy --all-targets`, `cargo fmt`, build without default features.
- [ ] CHANGELOG: one `### Fixed` entry.
- [ ] Fold delta.md into cairn/spec/backends.md; append the log entry; mark `landed`.
