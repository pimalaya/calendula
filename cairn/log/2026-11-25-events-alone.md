---
cairn: log
change: events-alone
landed: 2026-11-25
---

# Calendula serves events alone

## What landed

- The `item` family is removed from the CLI, the REPL, the MCP server (`item_*` tools) and `json-schema` (`calendula-item-*`), no hidden alias kept: `calendula item ...` exits 2 `usage-invalid`. Raw access to any component stays in `caldav get`, `caldav put`, `caldav report query --component` and `vdir item`.
- The capabilities `item.*` are `event.*`, and the codes `item-not-found`, `item-unreadable` and `item-invalid` are `event-*`; `item-duplicate` stays io-pimdir's.
- `item.composer` is `event.composer`; `item.list.page-size` is gone, both former keys refused by serde.
- `event read`, `event update` and `event delete` naming a resource that holds no VEVENT fail `event-not-found`, the resource untouched (`ensure_event`); before, `event delete` removed a to-do and `event update <id> <ICAL>` overwrote it. `event update` given a source reads the id first, as `event delete` does.
- `event add` and `event update` refuse a source holding no VEVENT `event-invalid` (`ensure_event_source`), so `event` writes no other component.
- The `ItemOrigin` plumbing and the pimdir foreign-body stripping are gone: every body is the user's, queued whole (`enqueue_user_body`). The `IcalFamily` split and the build pipeline's `NoFields` collapse to `event`.
- tests/event_kind.rs pins the mixed calendar (red first against the old code); the item cases of the other suites moved onto `event` or went.
- Spec: commands.md (one family, capability registry, codes, the id and write rules, the item requirements removed), backends.md (codes, pimdir queueing), config.md (the composer), mcp.md (the tools), repl.md (the composer refusal).
- Docs: config.sample.toml, README Features, the 1.0.0 changelog (a BREAKING removal naming the replacements, the renames folded into their entries), cardamum's spec mention of `item update`.
- Capabilities moved: `item.list`, `item.read`, `item.add`, `item.update`, `item.delete`, `item.update.if-match` and `item.delete.if-match` renamed to `event.*`.
