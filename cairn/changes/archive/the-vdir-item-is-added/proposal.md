---
cairn: change
id: the-vdir-item-is-added
status: landed
created: 2026-11-16
---

# The vdir item is added

> L18 of the CLI alignment plan, owner ruling D32 of 2026-11-16.

## Why

pimalaya-cli commands.md ("A verb keeps one meaning") gives `add` to creating an item and `create` to a collection. `vdir item create` stored an item, so it spelled the item verb as the collection one, beside the unified `item add` and `event add`. vdir is a local namespace, not a protocol one, so it takes the family verb.

## What

- src/vdir/item/create.rs becomes src/vdir/item/add.rs, `VdirItemCreateCommand` and `VdirItemCreateOutput` becoming `VdirItemAddCommand` and `VdirItemAddOutput`; the `VdirItemCommand::Create` variant becomes `Add`, keeping its visible alias `new` and taking `create` as a hidden one, so a script running `vdir item create` keeps working and the help, the manual and the completions name `add`. `vdir create` (a collection) stays.
- The JSON schema key follows the command path: `calendula-vdir-item-create` becomes `calendula-vdir-item-add`. `vdir item` never shipped (v0.2.0 has no `vdir item`), so nothing breaks: the CHANGELOG [Unreleased] Added line is reworded and MIGRATION.md gains nothing.
- The MCP server exposes no `vdir` command, so no tool name moves.
- Spec: commands ("vdir items").
