---
cairn: log
change: the-last-check-follow-ups
landed: 2026-11-15
---

# The last check's follow-ups

Rows M2, M8, L2, L6, L9, L10, L11 and L19 of X3, the last check of the family's CLI alignment round, land; L7 and L8 needed nothing. Capabilities **mcp** and **commands** moved.

- **src/mcp.rs**: `openWorldHint` read off the arguments a tool lists, so the item writes keep `false` under `--allow-send` (M2).
- **src/shared/event/expand.rs**: `startsAt` and `endsAt` spell a zero offset `Z` (M8).
- **src/vdir/list.rs**: `description`, `color` and the COLOR column (L2).
- **src/shared/item.rs, src/shared/item/read.rs, src/shared/event/read.rs, src/shared/build.rs, src/shared/event/build.rs**: a read always carries `contents` and `encoding`, `item list` alone printing them `null`; the builds print `encoding` and hand over bytes that are not UTF-8 (L6).
- **src/backend.rs, src/cli.rs, src/account/cli.rs, src/account/list.rs, src/gcal/cli.rs, src/pimdir/cli.rs, src/vdir/cli.rs**: the `-b` help and its possible values, the abouts (L9, L10, L11).
- **src/error.rs and the item and event read, update and delete commands**: one not-found sentence whatever the backend (L19).
- **L7**: `tag-refused` comes from io-pimdir's search tag alone, which calendula never calls: domain, as backends.md says. **L8**: cardamum's `-k` is `<ADDRESSBOOK-ID>`, the twin of `<CALENDAR-ID>`: already aligned.
- **Evidence**: on the code before, tests/mcp.rs `scheduling_is_listed_under_allow_send_as_well` and `a_gate_moves_no_hint_by_itself`, the expand and pimdir tests reading `Z`, tests/read.rs `an_unknown_id_is_coded_not_found` and `an_empty_item_reads_as_the_empty_text`, tests/caldav_id.rs `an_item_id_out_of_its_calendar_is_not_found`, src/shared/build.rs `a_built_item_names_its_encoding` and tests/help.rs fail.

CHANGELOG (Added, Changed), MIGRATION.md (the item row). Spec: mcp, commands.
