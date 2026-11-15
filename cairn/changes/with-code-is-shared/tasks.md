---
cairn: tasks
change: with-code-is-shared
---

# Tasks

- [x] Pin pimalaya-cli 51ce8b1; Cargo.lock moves its source line alone.
- [x] Delete `error::with_code` and its mechanics test; call the shared helper in `Command::execute`, the MCP exit, `error::unreadable` and the two msgraph tests.
- [x] Log, set `landed`.
