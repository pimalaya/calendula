---
cairn: log
change: an-option-after-a-query-is-an-option
landed: 2026-11-15
---

# An option after a query is an option

`pimdir search` and `pimdir count` read an option wherever it stands, where every word after the first was a query word, the defect the live testing found in Himalaya and cardamum. A `-` term goes after `--`. Capability **commands** moved.

- **src/pimdir/search.rs, src/pimdir/count.rs**: the query positional drops `trailing_var_arg` and `allow_hyphen_values`; its help names `--`.
- **Evidence**: src/cli.rs `an_option_after_a_query_is_an_option` failed on the code before, the query reading `x --page-size 3 --json` and `--json` unset.

CHANGELOG (Fixed), MIGRATION.md. Spec: commands ("pimdir searches calendar items through the store's index").
