---
cairn: change
id: vdir-home-dir-becomes-root
status: landed
created: 2026-11-15
---

# The vdir home is `vdir.root`

> Line K10 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), decision D13; cardamum's C10 lands the same change under the same id.

## Why

Every other store a Pimalaya app reads names its directory `root`: `pimdir.root` in the four pimdir readers, `store.root` in neverest. The vdir block alone spelled it `vdir.home-dir` (audit finding F39), in calendula and cardamum both. The owner ruled the twins move to `vdir.root` (D13, 2026-11-15).

## What

- `vdir.root` names the vdir home, the directory holding one calendar per immediate subdirectory, its `~` and environment variables expanded as it is read, as `pimdir.root`.
- `vdir.home-dir` is refused by name in the family sentence (pimalaya-config 0.5.1's `removed_key!`): ``The `vdir.home-dir` key is removed, the vdir home being named `root` as a store root is: rename it `vdir.root`, the path unchanged, see MIGRATION.md``, alone or beside `vdir.root`. A build without `vdir` loads the block unread, as before.
- config.sample.toml, README.md, the tests' configurations: `vdir.root`.
- Breaking: CHANGELOG (Changed, **Breaking**) and a MIGRATION.md entry under "From v0.2 to v0.3", before and after.
- Spec: config ("The vdir home is `vdir.root`", added; "Unknown fields", "Backend blocks" and "Paths are shell-expanded", modified).
