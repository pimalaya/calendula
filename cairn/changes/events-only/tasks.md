---
cairn: tasks
change: events-only
---

- [ ] Blocked on: unfog's task family and the journal CLI's entry family released, so the commands have a home
- [x] delta.md: MODIFIED "One family per stored component kind", "The component families against item", "Columns follow the kind", "Only events draw an agenda"; REMOVED the todo and journal parts; ADDED the leftover-table warning
- [ ] Remove src/shared/todo, src/shared/journal, their CLI wiring (src/cli.rs), `TodoConfig`, `JournalConfig`, the render order entries
- [ ] Leftover `todo` and `journal` tables ignored with a warning, a test for each
- [ ] Check the shared `calendar create` component set on each backend
- [ ] README, config.sample.toml, MIGRATION.md, CHANGELOG (Removed)
- [ ] `cargo fmt`, `cargo test`, `cargo clippy` through `nix develop --command`
- [ ] Fold into spec/commands.md and spec/config.md, log entry
