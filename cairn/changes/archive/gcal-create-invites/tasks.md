---
cairn: tasks
change: gcal-create-invites
---

# Tasks

- [x] `create_item`: an event with an `ATTENDEE` and no `ORGANIZER`, or the calendar's own address as `ORGANIZER`, goes through `events.insert` with `GcalSendUpdates::All`, its `iCalUID` in the body; the rest as today.
- [x] Tests: `invites` holds with no organizer and with the calendar's address (in another case), and not with another organizer or with no attendee.
- [x] Checked against Google with a second mailbox of mine: `event create --attendee` mails the invitation and the event keeps the `UID` it was built with.
- [x] `cargo test`, `cargo clippy --all-targets`, `cargo fmt`, build without default features.
- [x] CHANGELOG: one `### Fixed` entry.
- [x] Fold delta.md into cairn/spec/backends.md; append the log entry; mark `landed`.
