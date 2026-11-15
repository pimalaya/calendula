---
cairn: delta
change: the-kit-0-5-1-is-adopted
---

# Delta

## ADDED Requirements

### Requirement: A version line prints the version
(repl) A `-V` or `--version` line SHALL print the version the command line prints, `-V` the short one and `--version` the long one, the `Command` enum carrying `#[command(version, long_version = long_version!())]` for pimalaya-cli's loop to read; the session SHALL go on after it.

#### Scenario: The version in a text session
- GIVEN a text session
- WHEN the lines `-V` and `--version` are read
- THEN stdout holds `calendula <version>`, then `calendula v<version> <features>` and the build lines

## MODIFIED Requirements

### Requirement: Text output is plain text
(commands) `account list` SHALL print pimalaya-cli's `account::AccountListOutput` text, as Himalaya does: one line per account through `Columns` with no header row, its name, its backends (each the build leaves out as `<name> (not built)`, comma-separated), then `default` for the account a command with no `-a` runs against, `-` otherwise. It leaves the list of `Columns` outputs holding a header row. `account check` prints pimalaya-cli's `account::AccountCheckOutput`, its `Fields` unchanged.

### Requirement: An account names the backends left out of the build
(commands) The backends cell of the `account list` text, not a BACKENDS column, lists the built backends, then each left out as `<name> (not built)`, comma-separated.

### Requirement: Every data command returns a named output type
(commands) A `build` writing to `-o` SHALL print pimalaya-cli's `output::SavedOutput`, `{path, size}`, its text `Saved <size> bytes to <path>`.

### Requirement: A session refuses what cannot finish inside it
(repl) A `repl` or `mcp` line SHALL be refused by pimalaya-cli's loop before calendula runs anything, with `Cannot start a session inside a session`, coded `usage-invalid`; calendula keeps no refusal of its own for it.

### Requirement: A tool's input is the command's grammar
(mcp) A value the command's own parser refuses, or an argument clap finds missing, SHALL be a tool error coded `usage-invalid` built by the tool's `Grammar::usage_error`, naming the argument as the tool's input does: ``Invalid value `<value>` for `<name>`: <reason>``, ``Missing argument `<name>` ``, never clap's usage text.

### Requirement: Unknown fields
(config) A key or a table calendula no longer reads SHALL be refused through pimalaya-config's `removed_key!`, in the family sentence ``The `<name>` table is removed, <why>: delete it, see MIGRATION.md`` (`key` for a key).

### Requirement: A leftover todo or journal table is refused by name
(config) The refusal SHALL read ``The `todo` table is removed, the to-dos having moved to unfog, the Pimalaya CLI for tasks: delete it, see MIGRATION.md`` and ``The `journal` table is removed, the journal entries having moved to journal, the Pimalaya CLI for journal entries: delete it, see MIGRATION.md``.

### Requirement: No colour or table is configured
(config) Each refusal SHALL read ``The `<table>` table is removed, <command> printing plain text with no colour: delete it, see MIGRATION.md`` (`calendula printing plain text with no border or colour` for `table`); its scenario's error says the `event.list.table` table is removed.

## REMOVED Requirements
