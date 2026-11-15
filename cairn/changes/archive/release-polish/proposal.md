---
cairn: change
id: release-polish
status: landed
created: 2026-11-12
---

# Release polish

## Why

Change 11, the last of the-family-standards (Ortie's id, cardamum and comodoro using it too): the documents a release reader meets say what the code now does, before the family releases.

## What

- README.md on the guidelines' template: eight one-line Features (readme-008), the provider subsections nested in the table of contents, the RFC table in order with RFC 7986, Configuration naming the shipped sample, its sections, Ortie for the OAuth 2.0 tokens, the welcome and `-c` once per path, Usage pointing at `--help` with a short REPL and MCP subsection. Every command and flag named checked against a fresh debug build.
- MIGRATION.md: one 0.2 to next-release section folding the pieces the earlier changes added, written for a 0.2 user only (what 0.2 never shipped leaves it), with jq recipes for the greps the old tables allowed.
- CHANGELOG.md `[Unreleased]` netted against 0.2.0: superseded entries folded into their final state, duplicates gone, released sections untouched.
- CONTRIBUTING.md: the feature table as Cargo.toml has it, the clippy matrix of the `features` job as the check to run, the live suites, the patch rule as packaging states it.
- The src/main.rs header refreshed as the architecture document.
- .github/workflows/release-on-demand.yml: the `os` input defaults to an option it lists.
- SECURITY.md checked against cardamum, Himalaya and Ortie.

## Scope / non-goals

No behaviour moves. The version bump, the MIGRATION and CHANGELOG headings it renames, SECURITY.md's supported line and the release are the owner's.
