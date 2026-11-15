---
cairn: change
id: the-verbs-are-the-familys
status: landed
created: 2026-11-15
---

# The verbs are the family's

> Line K8 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), findings F25 and F18, decision D10; cardamum's C6 is the twin.

## Why

The owner ruled `add` for storing an item, as Himalaya's `message add`, and `search` over `find`, as Himalaya's `envelope search` (D10). calendula's `event find` also printed a `nextPage` it never filled and took no `--page-token`.

## What

- `event add` and `item add`, `create` a hidden alias of each; `event search <UID>`, `find` its hidden alias.
- `event search` takes `-p/--page`, `-s/--page-size` and `--page-token`, cutting its matches as `event list` cuts a page, a series and its overrides one resource; with no `-s` every match is one page, as before.
- The MCP tools follow the leaves: `event_add`, `item_add`, `event_search`, where they were `event_create`, `item_create`, `event_find`. A breaking change for an MCP client: MIGRATION.md says so. The JSON schemas are `calendula-event-add`, `calendula-item-add`, `calendula-event-search`.
- Spec: commands, mcp, repl, backends wherever the verbs are named.
