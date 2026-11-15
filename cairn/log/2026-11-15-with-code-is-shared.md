---
cairn: log
change: with-code-is-shared
landed: 2026-11-15
---

# with_code is shared

calendula's `error::with_code` is gone for pimalaya-cli's `error::with_code(err, code)` (51ce8b1, item 13 of the 2026-11-15 owner round). No capability moved.

- **Cargo.toml**: pimalaya-cli pinned to 51ce8b1; Cargo.lock moved its source line alone.
- **src/error.rs**: `with_code` and its mechanics test deleted; `error::unreadable` calls the shared helper with `error::code`. **src/cli.rs**: `Command::execute` and the MCP exit do the same. The msgraph tests of src/shared/event/delete.rs and src/msgraph/backend.rs, which assert calendula's codes through it, now call the shared helper.
- **Evidence of no behaviour change**: the shared helper differs from the old one only in leaving alone a chain coded `usage-invalid` or holding a `PartialOutcome`, which `error::code` never coded, so the old helper returned them unchanged too. Every test asserting a code through it (the msgraph refusal tests, tests/capability.rs, tests/mcp.rs, `account check` rows) passes unchanged.
