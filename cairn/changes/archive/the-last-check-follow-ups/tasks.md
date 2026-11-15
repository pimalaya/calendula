---
cairn: tasks
change: the-last-check-follow-ups
---

# Tasks

- [x] M2: src/mcp.rs reads `openWorldHint` off the arguments listed; tests/mcp.rs `scheduling_is_listed_under_allow_send_as_well` (item hints `false`) and `a_gate_moves_no_hint_by_itself`, both failing on the code before.
- [x] M8: src/shared/event/expand.rs `rfc3339` spells a zero offset `Z`; the expand, pimdir index and pimdir backend tests read `Z` (they read `+00:00`).
- [x] L2: src/vdir/list.rs `description`, `color` and the COLOR column; `a_row_carries_the_description_and_the_colour`.
- [x] L6: src/shared/item.rs `ItemContents` non-null, `ItemRow` keeping the listing's nulls, `ItemReadRow` for the reads; src/shared/item/read.rs, src/shared/event/read.rs; src/shared/build.rs and src/shared/event/build.rs print `encoding`, bytes that are not UTF-8 in base64 or raw; tests/read.rs `an_empty_item_reads_as_the_empty_text` and src/shared/build.rs `a_built_item_names_its_encoding`, failing before.
- [x] L9, L10, L11: src/backend.rs `parser`, src/cli.rs the `-b` help, src/account/cli.rs, src/account/list.rs, src/gcal/cli.rs, src/pimdir/cli.rs, src/vdir/cli.rs; tests/help.rs.
- [x] L19: src/error.rs `item_missing`, `event_missing`; the item and event read, update and delete commands; tests/read.rs `an_unknown_id_is_coded_not_found` and tests/caldav_id.rs `an_item_id_out_of_its_calendar_is_not_found` assert the sentences.
- [x] L7, L8: checked, nothing to change (proposal).
- [x] CHANGELOG, MIGRATION.md; fold the delta; log.
