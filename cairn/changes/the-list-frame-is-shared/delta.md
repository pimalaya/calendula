---
cairn: delta
change: the-list-frame-is-shared
---

# Delta

Every requirement lives in commands.md.

## ADDED Requirements

## MODIFIED Requirements

### Requirement: A command's errors print at its end
In the text output, each `errors` entry SHALL print as one `Error:` line, `<id>  <code>  <error>` with `-` for a null id or code, last in the command's output: after the items (rows, records or the agenda's lines) and one blank line, the `Error:` lines aligned among themselves as pimalaya-cli's `Columns` aligns cells, each value sanitized (pimalaya-cli `error::ItemErrors`). Nothing SHALL print, the blank line included, when `errors` is empty.

### Requirement: Text output is plain text
`event list` and `event find` SHALL print as Himalaya's `envelope list` does, through pimalaya-cli's `listing::Listing` (the rest of the paragraph unchanged). `account capability` leaves the list of `Columns` tables: it prints as "An account lists what it serves" says.

### Requirement: An account lists what it serves
`account capability` SHALL print pimalaya-cli's `capability::AccountCapabilityOutput`, reading the configuration and the tables and connecting nowhere: `{"account", "capabilities": [{"name", "backend", "level", "detail"}]}`, one row per capability of the unified API, `backend` the backend the unified commands resolve to (the first the account configures that `--backend` allows, in the order `auto` tries them), `level` one of `full`, `partial`, `server` and `none`, `detail` what the level leaves out or `null` for a full one. `--backend <name>` SHALL read that backend of the account. When the account configures no backend `--backend` allows, every row SHALL read `backend` `null` and level `none`, and the command SHALL succeed. The text SHALL be a pimalaya-cli `listing::Listing` headed by `Account: <name>`, then a blank line and one line per capability with no column header: the name, the backend, the level and the detail.

#### Scenario: A vdir account
- GIVEN the account `test` configuring a vdir home alone
- WHEN `calendula account capability` runs
- THEN it prints `Account:    test`, a blank line and fifteen rows naming `vdir`, each `full` but `item.update.if-match`, `item.remove.if-match`, `online-meeting` and `scheduling.send-updates`, each `none` with why

#### Scenario: A backend the account lacks
- GIVEN the account `test` configuring a vdir home alone
- WHEN `calendula --json --backend pimdir account capability` runs
- THEN every row has `"backend": null` and `"level": "none"`, and the exit status is 0

## REMOVED Requirements
