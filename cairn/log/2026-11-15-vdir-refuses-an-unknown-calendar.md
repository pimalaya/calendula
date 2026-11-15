---
cairn: log
change: vdir-refuses-an-unknown-calendar
landed: 2026-11-15
---

# vdir refuses an unknown calendar

Every vdir command naming a calendar its home does not hold fails `calendar-not-found`, exit 1, where `event list -k nope` exited 0 with an empty list, `event create -k nope` invented the calendar and `calendar delete -k nope` failed uncoded. Line K2 (F31) of the CLI alignment round. Capability **backends** moved.

- **src/vdir/client.rs**: `collection_dir` resolves an id to its directory only when the id is one path segment, neither `.` nor `..`, naming an existing directory; `VdirClient::collection_path` refuses another with ``Collection `<id>` not found``, which `vdir delete` and `vdir rename` call.
- **src/vdir/backend.rs**: `calendar_path` replaces the unchecked `path`, refusing with ``Calendar `<id>` not found`` in `update_calendar`, `delete_calendar`, `list_items`, `get_item`, `create_item`, `update_item` and `delete_item`; `create_calendar` joins the id to the home as before.
- **pimdir**: no gap, `known_collection` already guarding every item read and write.
- **Evidence**: on a scratch vdir, `calendar delete -k ../sibling` removed a directory beside the home and `calendar delete -k ""` the home itself, before this change. tests/calendar_not_found.rs runs the binary: ten unified commands naming `nope`, `calendar delete` and `event list` naming `""`, `.`, `..`, `../outside` and `personal/..`, `vdir delete` and `vdir rename` naming an unknown collection, each coded, exit 1, the tree left as it was; a calendar the home holds still lists. Built without the check (the path joined unlooked), the first three tests fail.

CHANGELOG (Fixed). Spec: backends ("vdir refuses an unknown calendar").
