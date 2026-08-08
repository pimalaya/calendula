---
cairn: change
id: adopt-cairn
status: landed
created: 2026-08-08
---

# Adopt Cairn, retire ARCHITECTURE.md

## Why

calendula carried a root ARCHITECTURE.md, the per-repo architecture document the org convention retired in favour of the crate header plus a cairn folder. Nothing recorded why a decision was taken, and nothing forced the written design to follow the code. The three landing changes in this release (dropping io-calendar, adding pimdir, rewriting the wizard) all change behaviour, so they need somewhere to be proposed, folded and logged.

## What

The Cairn structure at the repository root: cairn/spec for the living truth, cairn/changes for in-flight proposals, cairn/log for the dated history, plus cairn.toml, the vendored cairn/verify.sh conformance checker, and the AGENTS.md activation stanza.

ARCHITECTURE.md is redistributed rather than deleted: what described the crate's shape moves into the src/main.rs header, and what described a capability becomes one spec file. The initial spec set is backends, commands, config, wizard and packaging.
