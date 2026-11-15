---
cairn: change
id: the-repository-follows-the-family
status: landed
created: 2026-11-15
---

# The repository follows the family

> Line K17 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), findings F45 and F46.

## Why

The siblings hold their exit contract in tests/exit.rs, where calendula had tests/usage.rs; they carry a CLAUDE.md pointing at AGENTS.md and a "Where changes belong" section in CONTRIBUTING.md; and their MIGRATION.md headings read as Himalaya's "From v2 to v3".

## What

- tests/exit.rs: tests/usage.rs folded in, with a `--json` after `--` and the standard commands with no argument, as neverest's.
- CLAUDE.md (`@AGENTS.md`), CONTRIBUTING.md "Where changes belong" as cardamum's.
- MIGRATION.md: "From v0.2 to v0.3" and "From v0.1 to v0.2", calendula's coming major being 0.3.
- No behaviour moves; no spec changes.
