---
cairn: log
change: the-vdir-item-is-added
landed: 2026-11-16
---

# The vdir item is added

`vdir item create` is `vdir item add`, the family verb for creating an item (pimalaya-cli commands.md, L18, owner ruling D32 of 2026-11-16). Capability **commands** moved.

- **src/vdir/item/add.rs** (was create.rs), **src/vdir/item/cli.rs**: `VdirItemAddCommand` and `VdirItemAddOutput`; the `Add` variant keeps its visible alias `new` and takes `create` as a hidden one, so a script running `vdir item create` keeps working.
- **src/json_schema.rs**: the key `calendula-vdir-item-add`. `vdir item` never shipped (not in v0.2.0), so the CHANGELOG Added line is reworded and MIGRATION.md gains nothing. The MCP server exposes no `vdir` command.
- **Evidence**: `the_vdir_item_is_added` in src/cli.rs parses `add`, `create` and `new` to the same command and renders `vdir item --help` (`add` listed, `create` absent, `new` the one visible alias); with the variant named `create` again it fails parsing `vdir item add`.

CHANGELOG (Added, reworded). Spec: commands ("vdir items").
