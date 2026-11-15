---
cairn: tasks
change: vdir-refuses-an-unknown-calendar
---

# Tasks

- [x] Tests through the binary (tests/calendar_not_found.rs) on a scratch vdir: every unified command naming `nope` fails coded, exit 1, the tree untouched; `""`, `.`, `..`, `../outside` and `personal/..` are not found and remove nothing; `vdir delete` and `vdir rename` refuse an unknown collection; a calendar the home holds still lists. The first three failed on the old path (exit 0 with an empty list, an uncoded failure, directories removed).
- [x] `collection_dir` in src/vdir/client.rs, `VdirBackend::calendar_path` and `VdirClient::collection_path` on top of it.
- [x] pimdir checked: `known_collection` guards every item read and write.
- [x] CHANGELOG Fixed line.
- [x] Fold the delta into [backends](../../spec/backends.md); log.
