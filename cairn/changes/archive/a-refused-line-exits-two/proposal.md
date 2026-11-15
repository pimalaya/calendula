---
cairn: change
id: a-refused-line-exits-two
status: landed
created: 2026-11-13
---

# A refused line exits two

## Why

pimalaya-cli's change a-refused-line-exits-two (2026-11-13) moved the family exit contract: clap already exited 2 on a command line it refused, with its text on stderr whatever `--json` said, while the family gave 2 to partial outcomes too, so a script could not tell a typo from an `account check` with a failed backend, and a `--json` caller got no failure object. No calendula is released with the old numbering (the partial `account check` is unreleased), so the owner chose on 2026-11-13 to give 2 to clap and move partial outcomes to 3.

## What

- pimalaya-cli points at the local tree (`[patch.crates-io]`, `path = "../cli"`) until the change is pushed.
- `main` parses through `pimalaya_cli::clap::parse` instead of `Cli::parse`: a refused line exits 2 and, with a `--json` word before any `--`, prints the failure object coded `usage-invalid` on stdout.
- The MCP server turns a call its command refuses into a tool error through `pimalaya_cli::error::usage_error`, coded `usage-invalid`; pimalaya-cli codes its own grammar refusals (unknown argument, wrong type) the same way, and its REPL a refused line.
- A failed `account check` exits 3 (`PartialOutcome::default()`), its help, tests, spec, CHANGELOG and MIGRATION saying so; `usage-invalid` joins the code table and the `calendula-error` schema.

## Scope / non-goals

No binary-specific partial status above the default exists, so nothing else shifts. The test-only `try_parse_from` calls stay plain clap errors.
