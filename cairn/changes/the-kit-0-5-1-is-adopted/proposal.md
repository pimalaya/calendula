---
cairn: change
id: the-kit-0-5-1-is-adopted
status: landed
created: 2026-11-15
---

# The kit 0.5.1 is adopted

> The adoption lines and K11 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), its decisions D3 and D5, on pimalaya-cli 0.5.1 and pimalaya-config 0.5.1 (released 2026-11-15). cardamum takes the same in the same pass.

## Why

pimalaya-cli 0.5.1 carries what calendula kept its own copy of: the `account list` and `account check` outputs (`AccountListOutput`, `AccountRow`, `AccountCheckOutput`, `BackendCheck`), the file a build writes (`SavedOutput`), the refusal of a nested `repl` or `mcp` line, coded `usage-invalid` where calendula's own arm left it uncoded, a `--version` line in a session once the command enum carries the version, and a refused MCP call naming the argument as the tool does rather than as clap's usage text. Its `account list` text follows Himalaya: no header row, the default marked, where calendula printed `NAME BACKENDS DEFAULT` and `yes`. pimalaya-config 0.5.1 carries the family sentence for a removed key or table (`removed_key!`): calendula's refusals had three shapes ("is gone", "moved to", "is removed"), mostly without MIGRATION.md (F41).

## What

- Cargo.lock: pimalaya-cli and pimalaya-config 0.5.0 to 0.5.1, nothing else.
- `account list` prints `pimalaya_cli::account::AccountListOutput`; the JSON is unchanged, the text is the shared one: `<name>  <backends>  default|-`, no header.
- `account check` prints `pimalaya_cli::account::AccountCheckOutput<ErrorCode>`, each row built by `BackendCheck::new` over the chain `with_code` codes with calendula's reader; JSON and text unchanged.
- src/repl.rs drops its `repl`/`mcp` arm, dead now that the kit refuses the line first, coded `usage-invalid`.
- The `Command` enum carries `#[command(version, long_version = long_version!())]`, so `-V` and `--version` lines print the version in a session.
- `mcp`: a call clap refuses answers `tool.grammar.usage_error(&err)`: ``Invalid value `someday` for `from`: ...``, ``Missing argument `<name>` ``.
- `event build -o` and `item build -o` print the kit's `SavedOutput`: `{path, size}` unchanged, the text `Saved <size> bytes to <path>` where it was `<Event|Item> successfully written to <path>`.
- K11: every removed key or table (`todo`, `journal`, `table`, the top-level `account`, `calendar.list`, `event.list.table`, `item.list.table`, `caldav.discover`) is refused through `pimalaya_config::removed_key!`, the sentence "The `<name>` table is removed, <why>: delete it, see MIGRATION.md" (`key` and `name the server in caldav.server instead` for `caldav.discover`, whose sentence was already this one).
- Spec: commands, repl, mcp, config.
