---
cairn: log
change: event-attendee-add-remove
landed: 2026-10-29
---

# A field flag adds or removes one attendee

`EventFieldsArgs` in src/shared/event/fields.rs takes `--add-attendee [NAME:]ADDRESS` and `--remove-attendee ADDRESS`, each repeatable and refused by clap alongside `--attendee`, so `event build`, `event create` and `event update` all have them. `attendee_edit` reads the VEVENT's `ATTENDEE` addresses (the `mailto:` dropped in any case, lowercased) and refuses, naming the address, an added one already there, a removed one absent, and one given to both flags, before the event is touched; `edit_attendees` then drops the removed lines and writes the added ones after the last `ATTENDEE` left, or where a missing property lands when none is. Every other line keeps its bytes, the other attendees' `PARTSTAT`, `ROLE`, `RSVP` and `CN` included.

Verification: `cargo test`, `cargo clippy --all-targets` and `cargo fmt` with the default features, and a build with `rustls-ring,vdir` only.

Capabilities moved: **commands** (a field flag adds or removes one attendee, added; a field flag sets its property and touches nothing else, modified).
