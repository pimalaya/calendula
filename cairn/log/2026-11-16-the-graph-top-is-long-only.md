---
cairn: log
change: the-graph-top-is-long-only
landed: 2026-11-16
---

# The Graph top is long-only

`msgraph calendars list` and `msgraph events list` take `--top` long-only, as Himalaya's Graph listings do (L17, owner ruling 2026-11-16). Capability **commands** moved.

- **src/msgraph/calendars/list.rs**, **src/msgraph/events/list.rs**: `top` trades `short = 's'` for a hidden `short_alias = 's'`; `--help` and the completions list `--top <N>` alone, `-s 5` still reads as `--top 5`, so no script breaks and MIGRATION.md gains nothing.
- **Evidence**: `the_msgraph_top_is_long_only` in src/cli.rs renders both helps (no `-s,`, `--top <N>` present) and parses `-s 5` into `top: Some(5)`; with `short = 's'` restored it fails on the help printing `-s, --top <N>`.

CHANGELOG (Changed). Spec: commands ("msgraph family").
