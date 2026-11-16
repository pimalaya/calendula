---
cairn: tasks
change: the-vdir-item-is-added
---

# Tasks

- [x] `vdir item add` (visible alias `new`, hidden `create`); types and file renamed to `add`.
- [x] JSON schema key `calendula-vdir-item-add`.
- [x] Regression test in src/cli.rs (`the_vdir_item_is_added`): `add`, `create` and `new` parse to the same command, the help lists `add` and not `create`; fails on the old code.
- [x] CHANGELOG [Unreleased] Added line reworded (never shipped).
- [x] Fold the delta, log, set `landed`.
