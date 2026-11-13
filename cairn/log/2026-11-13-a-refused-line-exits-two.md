---
cairn: log
change: a-refused-line-exits-two
landed: 2026-11-13
---

# A refused line exits two

From pimalaya-cli's change a-refused-line-exits-two (2026-11-13, uncommitted in pimalaya-cli when this landed): exit 2 is clap's, a refused command line, and a partial outcome moves to 3. No calendula is released with the old numbering.

- Cargo.toml patches pimalaya-cli to `path = "../cli"` (it was the git pin 0fbe5dd), which also brings 8cd41db (a manual page per command, scalar MCP arguments); nothing in calendula needed a change for it. The pin goes back to a git rev once pimalaya-cli is pushed.
- `main` parses through `pimalaya_cli::clap::parse`. `calendula --json --nope calendar list` exits 2 and prints `{"code":"usage-invalid","error":"unexpected argument '--nope' found\n\nUsage: calendula --json\n\nFor more information, try '--help'.","sources":[],"backtrace":null}`, reading no configuration; without `--json`, clap's text on stderr, exit 2, as before.
- The MCP server maps a call's argv clap refuses through `pimalaya_cli::error::usage_error`: `event_list` with `from: "someday"` is a tool error coded `usage-invalid` where it carried `"code":null` and the `error: ` prefix. pimalaya-cli codes its own grammar refusals (an unknown argument, a wrong type) and a REPL line clap refuses the same way.
- A failed `account check` exits 3 (`PartialOutcome::default()`): its help, the architecture header in src/main.rs, tests/account_check.rs, the live suites' `json_check_failure` (tests/common.rs), the spec, CHANGELOG and MIGRATION. No other partial status exists.
- Tests: tests/usage.rs (`--json --nope` and plain `--nope`), seen failing on `Cli::parse` (exit 2 with an empty stdout); tests/mcp.rs `a_value_the_command_refuses_is_a_usage_invalid_tool_error`, seen failing on the plain `?`; `usage-invalid` asserted on the MCP unknown argument and the REPL's refused line; the `check` unit test reads `PartialOutcome::status` 3.

Capabilities moved: commands (cairn/spec/commands.md: "A refused command line exits 2" added; `usage-invalid` in the code table; "The JSON contract is described and versioned" and "account check fails when a backend fails" modified), mcp (cairn/spec/mcp.md: "A tool's input is the command's grammar"), repl (cairn/spec/repl.md: "A session runs the CLI over one account"), packaging (cairn/spec/packaging.md: "Live provider suites", exit 3).

Verified: `cargo fmt --check`; `cargo clippy --locked --all-targets -- -D warnings` and `cargo test --locked` over every feature set of the CI matrix (default, `--all-features`, the TLS and backend sets); `--all-features` ran 273 unit tests, tests/account_check.rs 3, capability 9, event_list 4, first_run 6, mcp 12, read 2, repl 6, usage 2; `json-schema calendula-error` admits `usage-invalid` beside the code vocabulary; `sh cairn/verify.sh`.

Spec correction: commands.md said a failure with no code prints no `code` field; it prints `"code": null`, as pimalaya-cli always did.
