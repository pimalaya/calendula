---
cairn: tasks
change: event-attendee-add-remove
---

# Tasks

- [ ] `--add-attendee` and `--remove-attendee` on `event update`, parsed as `--attendee` is.
- [ ] The edit: drop the named `ATTENDEE`, write the added ones after the last `ATTENDEE`, fold long lines; every other line byte for byte.
- [ ] The refusals, before anything is written: address already present (whatever the name), address absent, one address on both flags, either flag with `--attendee`; each naming the address or the flags.
- [ ] Tests: the two scenarios of delta.md; a removal keeping the others' `PARTSTAT` and bytes; case-insensitive and `mailto:` matching; each refusal leaving the event untouched; an item with several VEVENTs refused as for any field flag.
- [ ] Docs: the `--attendee` help points to the two flags.
- [ ] `cargo test`, `cargo clippy --all-targets`, `cargo fmt`, build without default features.
- [ ] CHANGELOG: one `### Added` entry.
- [ ] Fold delta.md into cairn/spec/commands.md; append the log entry; mark `landed`.
