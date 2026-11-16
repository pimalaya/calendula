---
cairn: change
id: the-graph-top-is-long-only
status: landed
created: 2026-11-16
---

# The Graph top is long-only

> L17 of the CLI alignment plan, owner ruling 2026-11-16.

## Why

`msgraph calendars list` and `msgraph events list` showed `-s, --top <N>`, where Himalaya's Graph listings take `--top` long-only. In the unified listings `-s` is the page size, so the same letter meant Graph's `$top` here.

## What

- src/msgraph/calendars/list.rs and src/msgraph/events/list.rs: `top` drops its visible `short = 's'` for a hidden `short_alias = 's'`, so `--help`, the manual and the completions name `--top <N>` alone and a script passing `-s 5` still reads as `--top 5`. Nothing breaks, so no MIGRATION.md entry.
- Spec: commands ("msgraph family").
