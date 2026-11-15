---
cairn: tasks
change: a-vdir-id-is-one-directory-name
---

# Tasks

- [x] Tests through the binary (tests/calendar_not_found.rs), the home moved to <T>/outer/home beside <T>/outer/keep holding a file: `personal/` and `personal/.` are not found on `calendar delete`, `event list`, `vdir delete` and `vdir rename`, removing nothing; `calendar create` and `vdir create` with `..`, `../keep`, `../escape` and `x/y` and `vdir rename personal` to `..` or `../escape` fail uncoded, writing nothing. Three of the five tests failed on the old code (`calendar delete -k personal/` exit 0 removing `personal`, `vdir delete personal/` the same, `calendar create ..` exit 0 writing out of the home).
- [x] `is_dir_name` in src/vdir/client.rs, `collection_dir` on top of it.
- [x] `create_calendar` (src/vdir/backend.rs) and `vdir create` (src/vdir/create.rs) refuse an id that is not one directory name.
- [x] CHANGELOG Fixed lines.
- [x] Fold the delta into [backends](../../../spec/backends.md); log.
