---
cairn: tasks
change: the-shared-repl-and-mcp
---

# Tasks

- [x] Cargo.toml: pimalaya-cli `repl` and `mcp`
- [x] src/session.rs, every command over a session, the unified commands on `&mut CalendarClient`
- [x] src/repl.rs, the session refusals, `stdin-taken` in src/error.rs and `check_stdin` in `read_source`
- [x] src/mcp.rs, the tools gated by `--allow-write`, `--allow-send` and the capability tables, paths closed, the sending guard in `CalendarClient`, `instructions`
- [x] The `mcp` branch under `exit_on_failure`, `SILENT` gaining `calendula-repl` and `calendula-mcp`
- [x] tests/repl.rs, tests/mcp.rs, tests/first_run.rs and the unit tests, each seen failing first
- [x] README, CHANGELOG, the src/main.rs header
- [x] cargo fmt, clippy over the feature matrix, `cargo test --all-features`, `json-schema -d` compared, an MCP session over a scratch vdir
- [x] Fold into the spec (repl.md, mcp.md, commands.md), log, land
