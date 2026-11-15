---
cairn: tasks
change: the-verbs-are-the-familys
---

# Tasks

- [x] src/shared/event/cli.rs, src/shared/item/cli.rs: `Add` (alias `create`), `Search` (alias `find`); src/repl.rs follows.
- [x] src/shared/event/find.rs: the paging flags and the cut over the matched resources.
- [x] src/capability.rs, src/mcp.rs, src/json_schema.rs: the leaves, the tools, their instructions and the schema names.
- [x] Tests failing on the code before: tests/event_list.rs `a_search_resumes_at_the_next_page_it_names` (no `search`, no `--page-token`), tests/write.rs on `event add` and `item add`; tests/mcp.rs on the tool names; tests/write.rs `create_still_adds` keeps the aliases.
- [x] CHANGELOG, MIGRATION; fold the delta; log.
