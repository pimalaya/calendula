---
cairn: delta
change: a-refused-line-exits-two
---

# Delta

## ADDED Requirements

### Requirement: A refused command line exits 2
(commands) The command line SHALL be parsed by pimalaya-cli's `clap::parse`: help and version print as clap prints them, and a command line clap refuses (an unknown flag or command, a missing argument, a bad value) SHALL exit 2 before any configuration is read. With a `--json` word before any `--`, it SHALL print on stdout the failure object coded `usage-invalid`, its `error` clap's message without its `error: ` prefix nor colour, and nothing on stderr; otherwise clap's rendering on stderr and nothing on stdout. Exit 1 stays a failure object and 3 a printed result that is not a full success (see "account check fails when a backend fails").

#### Scenario: A misspelt flag
- WHEN `calendula --json --nope calendar list` runs
- THEN it exits 2 and prints `{"code":"usage-invalid","error":"unexpected argument '--nope' found …","sources":[],"backtrace":null}` on one line
- AND `calendula --nope calendar list` exits 2, printing clap's `error: unexpected argument '--nope' found` on stderr and nothing on stdout

## MODIFIED Requirements

### Requirement: A JSON error carries a stable code
(commands) A failure a caller is expected to act on SHALL carry a stable code, printed under `--json` as a `code` string beside `error`, `sources` and `backtrace`. The code SHALL be found anywhere in the error chain, so added context does not hide it. A failure with no code SHALL print no `code` field. The plain output SHALL stay unchanged. Codes are kebab-case and never renamed; `error` stays free wording.

| Code | Raised when |
|---|---|
| `capability-missing` | the backend the account resolves to does not serve a capability the command reaches; nothing was read, opened or connected |
| `stdin-taken` | a command run inside a session (`repl`, `mcp`) would read its iCalendar from stdin, which carries the session's commands; nothing was read |
| `body-pending` | a listed item's body is not local yet (pimdir); a sync brings it |
| `precondition-failed` | a write's `--if-match` names a version the item no longer has (pimdir) |
| `occurrence-unsupported` | `pimdir reply` or `pimdir cancel` names an occurrence its performer cannot act on alone |
| `item-duplicate` | a pimdir create names a `UID` its calendar holds or has queued |
| `store-uncreated` | the pimdir store has no schema yet: its sync engine creates it |
| `store-version` | the pimdir store is of another format version than calendula's: an earlier sync engine's is reinstalled, a newer one's needs calendula upgraded |
| `store-foreign` | the pimdir database is not a pimdir store: another program wrote it |
| `store-corrupt` | the pimdir store's two version stamps disagree, or its schema is not the canonical one |
| `store-hash-algo` | the pimdir store names its bodies by a hash this calendula does not know: upgrade calendula |
| `store-busy` | another process holds the pimdir store; retry |
| `store-exposed` | a path of the pimdir store is a link or another user can reach it: restrict it |
| `lock-replaced` | the lock file of the pimdir store was removed or replaced under a write: nothing was queued; retry |
| `flag-invalid` | a pimdir write carries a flag the store cannot hold |
| `action-malformed` | a pimdir queue action's payload is one its schema refuses |
| `index-missing` | the pimdir store has no search index yet: its sync engine builds it |
| `index-outdated` | the pimdir search index was built otherwise or from another store: its sync engine rebuilds it |
| `index-version` | the pimdir search index is of another draft or shape: its sync engine rebuilds an earlier draft's, and leaves any other |
| `index-zone-unknown` | the pimdir search index reads floating times, or the search reads days, in a zone the time zone database does not know |
| `index-fts5-missing` | the SQLite calendula links lacks FTS5 or is older than 3.43 |
| `query-refused` | the pimdir search query is malformed or names an unknown field or value |
| `query-unanswerable` | the pimdir search index could not answer an accepted query |
| `usage-invalid` | the command line does not match the grammar (an unknown flag or command, a missing argument, a bad value), raised by pimalaya-cli; nothing ran |

#### Scenario: A stale version
- GIVEN a pimdir event whose body changed since it was read
- WHEN `calendula --json event delete --if-match <old etag>` names it
- THEN the command exits 1, queues nothing and prints `{"code":"precondition-failed","error":"Precondition failed: …","sources":[],"backtrace":null}`

### Requirement: The JSON contract is described and versioned
(commands) `json-schema` SHALL describe the failure object under the key `calendula-error`, its `code` typed by the code vocabulary or `usage-invalid`, the code pimalaya-cli raises itself. Every generated schema SHALL carry an `$id` naming the contract version: the major version of calendula, or `0.<minor>` before 1.0. Every registered payload SHALL be a JSON object, checked by a test. Piped `--json` output SHALL print one value per line.

### Requirement: account check fails when a backend fails
(commands) `account check` SHALL exercise every backend of the account `--backend` allows and print one verdict per backend: under `--json`, `ok`, the stable `code` of a failure (the code vocabulary of "A JSON error carries a stable code", `null` when the backend passed or its failure carries none) and the error of a failed one; the text shows no code. When any fails, it SHALL still print the report and exit 3, the family's partial outcome (`pimalaya_cli::error::PartialOutcome`), so a script reads the outcome from the exit code alone and which backends passed from the report; when all pass, it SHALL exit 0. It mirrors Himalaya's `account check`.

#### Scenario: One backend of two fails
- GIVEN the account `test` whose vdir home exists and whose pimdir root does not
- WHEN `calendula account check` runs
- THEN it prints `Account:  test`, `vdir:     OK` and `pimdir:   FAIL (<why>)`, and exits 3
- AND `calendula --json account check` prints the same report as its result, the pimdir verdict carrying `"code":"store-uncreated"` and the vdir one `"code":null`, and exits 3, and `calendula --backend vdir account check` exits 0

### Requirement: A session runs the CLI over one account
(repl) `calendula repl` SHALL start a session bound to one account, its configuration and one `--backend`, and run pimalaya-cli's shared REPL loop over it: read one command per stdin line, split as a shell splits it, a blank line or a line whose first word starts a `#` comment skipped, parse it with the one-shot grammar plus `--json`, subcommand names inferred from a unique prefix, and run it, until `quit`, `exit` or EOF. A line whose quote is not closed, a line that is not UTF-8, a parse error or a command error SHALL be printed, on stdout as the failure object when piped, a parse error's coded `usage-invalid`, and the loop SHALL continue. A missing configuration or account SHALL fail the session before its first line.

### Requirement: A tool's input is the command's grammar
(mcp) A tool's `inputSchema` SHALL be generated from its clap command by pimalaya-cli's `mcp::grammar::Grammar`, each property named as the CLI spells the argument, with `additionalProperties: false`, and a call SHALL be parsed back by that same command from the argv the grammar writes. The composer arguments (`interactive`, `composer`) SHALL be no property of any tool. An argument the `inputSchema` does not declare SHALL be refused before the command runs as a tool error ``Unknown argument `<key>`, <tool> takes <properties>``, and a value of the wrong type the same way, both coded `usage-invalid`; a value the command's own parser refuses (a `from` that is no date) SHALL be a tool error coded `usage-invalid` too, built by pimalaya-cli's `error::usage_error`; an unlisted tool SHALL be a JSON-RPC error -32602.

### Requirement: Live provider suites
(packaging) The binary SHALL carry one ignored integration suite per provider under tests/ (google.rs for the Google Calendar API and Google CalDAV, microsoft.rs for Microsoft Graph, fastmail.rs for Fastmail CalDAV), run with `--ignored` and sharing tests/common.rs. Each SHALL hold an `anonymous` module that runs every backend the provider's sample sections offer on a made-up credential and touches no account, its accounts taken from the provider sections of config.sample.toml so a provider drifting from the sample fails it, and asserting the exit code of the output contract (3 with the `account check` report, 1 with the failure object) and a message naming the refusal in calendula's words (`HTTP 401`). It MAY hold a `credentialed` module that mints its own token from `CALENDULA_`-prefixed variables and only reads (`account check`, `calendar list`, `event list`), failing naming the variables it misses, which CI runs in the upstream repository only and once its secrets exist; Google's SHALL ask `calendar.events` and `calendar.calendarlist.readonly` alone, and so covers the Google Calendar API only. A token or password SHALL reach the binary through a command reading its environment, never argv nor the configuration, and no assertion SHALL print a token, a secret or an event's value. CI SHALL run each suite in its own job at `RUST_LOG=debug` at most.

## REMOVED Requirements

None.
