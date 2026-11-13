---
cairn: tasks
change: a-refused-line-exits-two
---

# Tasks

- [x] Cargo.toml: pimalaya-cli patched to `path = "../cli"`.
- [x] src/main.rs: `pimalaya_cli::clap::parse` replaces `Cli::parse`; the architecture header names exits 2 and 3.
- [x] src/mcp.rs: a call's argv clap refuses goes through `usage_error`.
- [x] src/account/check.rs: help says status 3; the unit test reads `PartialOutcome::status` 3.
- [x] tests/usage.rs: `--json --nope` prints the `usage-invalid` object on stdout and exits 2, seen failing on `Cli::parse`; without `--json` clap's text on stderr, exit 2.
- [x] tests/mcp.rs: a `from` that is no date is a tool error coded `usage-invalid`, seen failing on the plain clap error; an unknown argument is coded too.
- [x] tests/repl.rs: a line clap refuses answers `usage-invalid`.
- [x] tests/account_check.rs and tests/common.rs: a partial outcome exits 3.
- [x] Fold the delta into the spec (commands, mcp, repl, packaging); CHANGELOG; MIGRATION; log.
- [x] fmt; clippy `--all-targets -D warnings` and tests over every CI feature set; `sh cairn/verify.sh`.
