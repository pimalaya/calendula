---
cairn: log
change: the-kit-0-5-1-is-adopted
landed: 2026-11-15
---

# The kit 0.5.1 is adopted

calendula moves to pimalaya-cli 0.5.1 and pimalaya-config 0.5.1 and drops its own copies of what they now carry, the adoption lines and K11 of the CLI alignment round. Capabilities **commands**, **repl**, **mcp** and **config** moved.

- **Cargo.lock**: pimalaya-cli and pimalaya-config 0.5.0 to 0.5.1, nothing else.
- **src/account/list.rs, src/account/check.rs, src/json_schema.rs**: `account list` and `account check` print `pimalaya_cli::account::{AccountListOutput, AccountCheckOutput}`, each check row `BackendCheck::new` over the chain `with_code` codes; the JSON is unchanged, and the `account list` text is the shared one, `<name>  <backends>  default|-` with no header row, where it printed `NAME BACKENDS DEFAULT` and `yes`.
- **src/repl.rs, src/cli.rs**: the nested-session arm goes, the kit refusing a `repl` or `mcp` line first, coded `usage-invalid` where calendula's refusal was uncoded; the `Command` enum carries the version, so `-V` and `--version` lines print it.
- **src/mcp.rs**: a call clap refuses answers `Grammar::usage_error`, ``Invalid value `someday` for `from`: ...``, where it answered clap's `invalid value 'someday' for '--from <DATE>'`.
- **src/shared/build.rs**: `build -o` prints the kit's `SavedOutput`, the text `Saved <size> bytes to <path>`.
- **src/config.rs**: the eight removed keys and tables refused through `removed_key!`, "The `<name>` table is removed, <why>: delete it, see MIGRATION.md", where calendula had three shapes, most without MIGRATION.md.
- **Evidence**: on the code before, tests/account.rs `the_account_list_is_the_shared_text` read the header row, tests/repl.rs read `code: null` on a nested line and a refused `-V`, tests/mcp.rs read clap's `'someday'`, tests/write.rs read `Item successfully written to`, and the config tests read "is gone" and "moved to".

CHANGELOG (Changed). Spec: commands, repl ("A version line prints the version", added), mcp, config.
