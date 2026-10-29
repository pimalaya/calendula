---
cairn: tasks
change: event-attendee-add-remove
---

# Tasks

- [x] `--add-attendee` and `--remove-attendee` on `event build`, `event create` and `event update` (the shared field flags), parsed as `--attendee` is.
- [x] The edit: drop the named `ATTENDEE`, write the added ones after the last `ATTENDEE`, fold long lines; every other line byte for byte.
- [x] The refusals, before anything is written: address already present (whatever the name), address absent, one address on both flags, either flag with `--attendee`; each naming the address or the flags.
- [x] Tests: the two scenarios of delta.md; a removal keeping the others' `PARTSTAT` and bytes; case-insensitive and `mailto:` matching; each refusal leaving the event untouched; an item with several VEVENTs refused as for any field flag; `event build` editing a source.
- [x] Docs: the `--attendee` help points to the two flags.
- [x] `cargo test`, `cargo clippy --all-targets`, `cargo fmt`, build without default features.
- [x] CHANGELOG: one `### Added` entry.
- [x] Fold delta.md into cairn/spec/commands.md; append the log entry; mark `landed`.
