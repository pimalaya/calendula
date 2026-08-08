---
cairn: log
change: adopt-cairn
landed: 2026-08-08
---

# Adopt Cairn, retire ARCHITECTURE.md

Added the Cairn structure at the repository root: cairn.toml, cairn/spec, cairn/changes, cairn/log, the vendored cairn/verify.sh, and the AGENTS.md activation stanza copied from himalaya.

Deleted the root ARCHITECTURE.md, the per-repo architecture document the org convention had already retired. Its content was redistributed rather than dropped: the crate's shape (where calendula sits in the stack, the three command families, the event-against-item split, backend selection, configuration and output) is now the src/main.rs header, which is what the guidelines make the architecture document of a binary. Everything that described a capability became one spec file.

The initial spec set is backends, commands, config, wizard and packaging, written against calendula as it stands after this release rather than as it stood before it, so the three behaviour changes landing alongside fold into it directly.

Spec updated: backends, commands, config, wizard, packaging (all ADDED).
