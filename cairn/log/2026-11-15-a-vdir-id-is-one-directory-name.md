---
cairn: log
change: a-vdir-id-is-one-directory-name
landed: 2026-11-15
---

# A vdir id is one directory name

A vdir calendar id holding a trailing separator (`personal/`, `personal/.`) names no calendar, and `calendar create` and `vdir create` refuse a new id that is not one directory name, where `calendar delete -k personal/` removed the calendar, `calendar delete -k personal/.` its events, and `calendar create .. <name>` wrote out of the home. Line K18 of the CLI alignment round, closing what vdir-refuses-an-unknown-calendar (2fc464f) left, as cardamum's 1d047d6 does. Capability **backends** moved.

- **src/vdir/client.rs**: `is_dir_name` (one normal path component that is the whole id: not empty, `.` nor `..`, no separator, a trailing one included), `collection_dir` on top of it, so every vdir command resolving a calendar refuses such an id `calendar-not-found`.
- **src/vdir/backend.rs**: `create_calendar` refuses an id that is not one directory name, ``Calendar id `<id>` is not one directory name``, uncoded.
- **src/vdir/create.rs**: `vdir create` refuses it, ``Collection id `<id>` is not one directory name``, uncoded. io-vdir 0.3 already refuses the new name of `vdir rename`.
- **Evidence**: tests/calendar_not_found.rs now runs over a home at <T>/outer/home beside <T>/outer/keep holding a file. On the old code `a_calendar_reaching_out_of_the_home_is_not_found` failed on `calendar delete -k personal/` (exit 0, `personal` gone), `the_vdir_commands_refuse_an_unknown_collection` on `vdir delete personal/` (exit 0) and `a_new_id_reaching_out_of_the_home_is_refused` on `calendar create ..` (exit 0, a `displayname` written beside the home); the old binary on scratch homes also emptied `personal` on `calendar delete -k personal/.` and made a directory beside the home on `vdir create ../<id>`.

CHANGELOG (Fixed). Spec: backends ("vdir refuses an unknown calendar", modified).
