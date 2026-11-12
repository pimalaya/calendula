---
cairn: tasks
change: wizard-runs-when-unconfigured
---

# Tasks

- [x] `configure` subcommand, aliased `wizard`
- [x] `offer_configuration`, raised from a bare invocation and from a command needing an account
- [x] `meet_bare_invocation`: offer only when unconfigured, not JSON, on a terminal, no `--account`
- [x] `discover::run` returns the account; saving, appending and printing move to `configure`
- [x] `AccountConfig::render`, name suffixing, default claiming
- [x] A build with no wizard-capable backend says so
- [x] cargo fmt, clippy, tests, feature matrix
- [x] Fold the delta and log
