---
cairn: tasks
change: events-only
---

- [x] delta.md: MODIFIED "One family per stored component kind", "The component families against item", "Columns follow the kind", "Only events draw an agenda", "A component window is applied locally", "A delete can be gated on a version", "A minted item carries an identity, and its family names its kind", "An event build answers its projection", "pimdir searches calendar items through the store's index", "Backend blocks", "The composer is a command, not a library", "A queued write is reported as queued"; ADDED "A leftover todo or journal table is refused by name"
- [x] Remove src/shared/todo, src/shared/journal, their CLI wiring (src/cli.rs), `TodoConfig`, `JournalConfig` and their list and table types, the account getters, the `json-schema` entries, the render order entries
- [x] pimdir search and count: `--kind` gone with the to-do and journal hits, events alone
- [x] `item build` and `item create` given no source name `calendula event`, `unfog` and `journal`
- [x] Leftover `todo` and `journal` tables refused by name, a test for each (top level and account), seen failing first
- [x] Check the shared `calendar create` component set on each backend (left as is, see the proposal)
- [x] README, config.sample.toml, MIGRATION.md, CHANGELOG (Removed), src/main.rs header
- [x] `cargo fmt`, `cargo test --all-features`, `cargo clippy --all-targets -D warnings` (default, all features, `rustls-ring` with each backend alone) through `nix develop --command`
- [x] Fold into spec/commands.md, spec/config.md and spec/backends.md, log entry
