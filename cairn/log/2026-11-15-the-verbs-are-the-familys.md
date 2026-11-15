---
cairn: log
change: the-verbs-are-the-familys
landed: 2026-11-15
---

# The verbs are the family's

`event add`, `item add` and `event search` replace `create` and `find`, kept as hidden aliases, and `event search` pages by the `nextPage` it prints, line K8 of the CLI alignment round (D10). Capabilities **commands**, **mcp**, **repl** and **backends** moved.

- **src/shared/event/cli.rs, src/shared/item/cli.rs, src/repl.rs**: the `Add` and `Search` leaves, `create` and `find` hidden aliases.
- **src/shared/event/find.rs**: `-p`, `-s`, `--page-token`; the matches cut by resource, `nextPage` set while one follows; no `-s` lists every match, as before.
- **src/capability.rs, src/mcp.rs, src/json_schema.rs**: the leaves `event search`, `event add`, `item add`; the tools `event_search`, `event_add`, `item_add` (breaking for MCP clients); the schemas `calendula-event-search`, `calendula-event-add`, `calendula-item-add`.
- **Evidence**: on the binary before, tests/event_list.rs `a_search_resumes_at_the_next_page_it_names` and tests/write.rs `every_write_receipt_says_whether_it_is_queued` (now on `event add`) failed.

CHANGELOG (Changed), MIGRATION.md. Spec: commands, mcp, repl, backends.
