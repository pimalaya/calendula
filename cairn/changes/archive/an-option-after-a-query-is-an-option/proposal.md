---
cairn: change
id: an-option-after-a-query-is-an-option
status: landed
created: 2026-11-15
---

# An option after a query is an option

> Found by the live testing of 2026-11-15 in Himalaya and cardamum, added to line K7's batch of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15); calendula shares it.

## Why

The query of `pimdir search` and `pimdir count` took `trailing_var_arg` and `allow_hyphen_values`, so every word after the first was a query word: `pimdir search x --page-size 3` searched `x --page-size 3`, a trailing `--json` printed text, and `-k work` after the query searched it.

## What

- The query is a plain repeated positional: an option is read wherever it stands, `--json` included.
- A query word starting with `-`, a negated term (`-lisbon`), goes after `--`, the options before it; before `--` it is refused as an unknown option, exit 2. The help says so.
- Breaking for a script writing a `-` term without `--`: MIGRATION.md says so.
- Spec: commands ("pimdir searches calendar items through the store's index").
