---
cairn: delta
change: the-shared-repl-and-mcp
---

# Delta

Folded into cairn/spec/repl.md and cairn/spec/mcp.md (new) and cairn/spec/commands.md on 2026-11-12.

## ADDED Requirements

### Requirement: A session runs the CLI over one account
(repl) `calendula repl` SHALL start a session bound to one account, its configuration and one `--backend`, and run pimalaya-cli's shared REPL loop over it: read one command per stdin line, split as a shell splits it, a blank line or a line whose first word starts a `#` comment skipped, parse it with the one-shot grammar plus `--json`, subcommand names inferred from a unique prefix, and run it, until `quit`, `exit` or EOF. A line whose quote is not closed, a line that is not UTF-8, a parse error or a command error SHALL be printed, on stdout as the failure object when piped, and the loop SHALL continue. A missing configuration or account SHALL fail the session before its first line.

### Requirement: A session connects once
(repl) Every command SHALL run over a session holding the unified client, built the first time a unified command needs it and kept for the commands after it. A command that fails SHALL drop the client it ran on, so the next command builds it again; the failed command SHALL NOT be retried. The protocol-specific commands SHALL build their own client each time. A one-shot run SHALL use a fresh session for its single command.

### Requirement: A session refuses what cannot finish inside it
(repl) Inside a session, `repl` and `mcp` SHALL be refused with `Cannot start a session inside a session`, and the composer (`-i`, with or without `--composer`, on `event build`, `event create`, `event update`, `item build`, `item create` and `item update`) SHALL be refused by name, its editor needing the terminal the session reads its commands from. The session SHALL go on after each.

### Requirement: A session never reads command input from stdin
(repl) A command run inside the session that would read its input from stdin (an iCalendar source given as `-`) SHALL be refused with the code `stdin-taken`, its input given as text instead.

#### Scenario: An event given as stdin
- GIVEN a piped session started with `--json`
- WHEN the line `event create -` is followed by `event list`
- THEN the first answers the failure object with `code` `stdin-taken` and the second lists the calendar, the event not created

### Requirement: A piped session answers once per line
(repl) When stdin is not a terminal, every line SHALL produce one flushed answer on stdout: the result, or the failure object (`calendula-error` under `--json`). Nothing SHALL be added after a result, every text output ending with its own newline. When stdin is a terminal, the prompt `calendula> ` SHALL render on stderr, a line clap cannot parse SHALL print clap's rendering on stderr, and a failed command SHALL print the failure object on stdout.

### Requirement: A line switches itself to JSON
(repl) A line carrying `--json` SHALL print its answer as JSON, the lines after it keeping the session's format. `--json` given to `repl` SHALL make every line JSON.

#### Scenario: One JSON line in a text session
- GIVEN a text session
- WHEN the lines `event list --json` and `calendar list` are read
- THEN the first prints one JSON value and the second the text list

### Requirement: Session flags are refused by name
(repl) A line clap cannot parse that sets a flag the session fixes when it starts, `-a`, `--account`, `-c`, `--config`, `--log-level`, `--log`, `--log-file`, `-b` or `--backend`, SHALL fail with ``Cannot set `<flag>` inside a session, start a new session with it instead`` in place of clap's error, the flag recognised alone, as `--<long>=<value>` or as a short flag with its value attached, before any `--`.

### Requirement: Help keeps JSON lines whole
(repl) A line asking for help SHALL print clap's text on stdout in a text session, and on stderr when the session's `--json` or a `--json` word of the line before any `--` is set, so stdout keeps one JSON value per line. Its usage line SHALL name the binary before the command, as `Usage: calendula event list [OPTIONS]`.

### Requirement: An MCP server over stdio
(mcp) `calendula mcp` SHALL run an MCP server (revision 2025-06-18) over stdio for one account and one `--backend` on pimalaya-cli's shared MCP core, answering `initialize`, `ping`, `tools/list` and `tools/call`, one JSON-RPC 2.0 message per line, with stdout carrying protocol messages only. A line that is not UTF-8 or not JSON SHALL answer -32700 and one that is not a JSON object, a batch included, -32600, both with a null `id`; a request whose `jsonrpc` is not `"2.0"` SHALL answer -32600 with its `id`; the server SHALL read on after each. Its commands SHALL run over a session kept for the server's lifetime, as the REPL's do, and SHALL never read their input from stdin. A failure of the server itself, resolving its account included, SHALL print as text on stderr and exit 1, leaving stdout empty, a person missing a configuration meeting the welcome instead (cairn/spec/config.md).

#### Scenario: A batch
- GIVEN a server
- WHEN the line `[{"jsonrpc":"2.0","id":1,"method":"ping"}]` is read, then a `ping` with id 4
- THEN the first answers -32600 with `id` `null` and the second `{}`

### Requirement: The server tells the agent how to use it
(mcp) `initialize` SHALL carry `instructions` saying how to find a calendar (`calendar_list`, the `calendar` argument, the default one when left out) and an event id (`event_list`, its pages and its window, `event_find` by UID), that an event is written from field arguments or iCalendar text and never from a path, that `account_capability` says what the backend serves, that `--allow-write` gates the writes and `--allow-send` the scheduling arguments and any write or delete of an item naming an `ATTENDEE`, and that a failed call answers `isError` with the `calendula-error` object. Every tool name it holds SHALL be one of the tool table.

### Requirement: Tools are the classified unified API
(mcp) The server SHALL expose one tool per classified leaf of the unified API, named `<group>_<leaf>`: `account_capability`, `calendar_list`, `event_list`, `event_read`, `event_find`, `item_list` and `item_read` read; `calendar_create`, `calendar_update`, `calendar_delete`, `event_create`, `event_update`, `event_delete`, `item_create`, `item_update` and `item_delete` write. `event build` and `item build`, `event agenda` and the protocol-specific commands SHALL NOT be exposed. A unified leaf SHALL be either a tool or explicitly left out, checked by a test.

### Requirement: Tools follow the capabilities
(mcp) A tool whose verb the account's backend declares `none` (cairn/spec/commands.md, Capability registry) SHALL NOT be listed, as the tables know without connecting: a server over a pimdir store lists no `calendar_create`, `calendar_update` or `calendar_delete`. A tool whose verb is served SHALL stay listed with all its arguments, one the backend does not serve answering `capability-missing` as a tool error.

### Requirement: A tool's input is the command's grammar
(mcp) A tool's `inputSchema` SHALL be generated from its clap command by pimalaya-cli's `mcp::grammar::Grammar`, each property named as the CLI spells the argument, with `additionalProperties: false`, and a call SHALL be parsed back by that same command from the argv the grammar writes. The composer arguments (`interactive`, `composer`) SHALL be no property of any tool. An argument the `inputSchema` does not declare SHALL be refused before the command runs as a tool error ``Unknown argument `<key>`, <tool> takes <properties>``, and a value of the wrong type the same way; an unlisted tool SHALL be a JSON-RPC error -32602.

### Requirement: A tool's output is the command's payload
(mcp) A successful call SHALL return the command's JSON value as `structuredContent` and as text, described as `outputSchema` by its registered schema when that schema's root is typed `object`. `event_create`, `event_update`, `item_create` and `item_update`, whose schema is one of two shapes (the write, or an abandoned composer edit), SHALL carry no `outputSchema`. A refused or failed call SHALL return `isError: true` with the `calendula-error` object and no `structuredContent`.

### Requirement: Writing and sending are opt-in
(mcp) Without flags the server SHALL list reading tools only. `--allow-write` SHALL add the calendar, event and item writes, without the scheduling arguments. `--allow-send` SHALL add the scheduling arguments, `attendee`, `add-attendee`, `remove-attendee`, `organizer` and `send-updates`, and SHALL require `--allow-write`, every write reaching people being a write first.

### Requirement: A write reaches attendees under --allow-send alone
(mcp) Inside a server without `--allow-send`, a create or an update SHALL be refused when the iCalendar it would write names an `ATTENDEE` in any component, an alarm's included, bytes that do not parse counting as naming one; and an update or a delete SHALL be refused when the stored item names one, read before the write. The refusal SHALL name `--allow-send`, and nothing SHALL be written; a document naming an attendee SHALL be refused before anything is read or connected. A backend, a server or a sync may notify the attendees an item names of its creation, its change or its deletion, which is why the rule holds on every backend, a local store included.

#### Scenario: Cancelling a meeting
- GIVEN a server started with `--allow-write` alone, over a calendar holding an event inviting `bob@example.org`
- WHEN `event_delete` is called on that event
- THEN the call answers `isError` naming `--allow-send`, and the event is still stored

### Requirement: A tool reads no local file
(mcp) Inside the MCP server, an iCalendar source SHALL be taken as text, a local path naming a file being no way in, and a source given as `-` SHALL be refused with `stdin-taken`.

### Requirement: Tool annotations say what a tool does
(mcp) A reading tool SHALL be `readOnlyHint`; an update or a delete SHALL be `destructiveHint`. `openWorldHint` SHALL be true exactly when the tool's effect may reach people beyond the account: on `event_create`, `event_update`, `event_delete`, `item_create`, `item_update` and `item_delete` under `--allow-send`, and on no tool otherwise.

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
| `flag-invalid` | a pimdir write carries a flag the store cannot hold |
| `action-malformed` | a pimdir queue action's payload is one its schema refuses |
| `index-missing` | the pimdir store has no search index yet: its sync engine builds it |
| `index-outdated` | the pimdir search index was built otherwise or from another store: its sync engine rebuilds it |
| `index-version` | the pimdir search index is of another draft or shape: its sync engine rebuilds an earlier draft's, and leaves any other |
| `index-zone-unknown` | the pimdir search index reads floating times, or the search reads days, in a zone the time zone database does not know |
| `index-fts5-missing` | the SQLite calendula links lacks FTS5 or is older than 3.43 |
| `query-refused` | the pimdir search query is malformed or names an unknown field or value |
| `query-unanswerable` | the pimdir search index could not answer an accepted query |

#### Scenario: A stale version
- GIVEN a pimdir event whose body changed since it was read
- WHEN `calendula --json event delete --if-match <old etag>` names it
- THEN the command exits 1, queues nothing and prints `{"code":"precondition-failed","error":"Precondition failed: …","sources":[],"backtrace":null}`

### Requirement: Nested execute
(commands) Each subcommand SHALL be a clap-derived struct carrying its own arguments, with an `execute(self, printer, client)` method, a unified command borrowing the client (`&mut CalendarClient`).

`Command::execute(printer, session)` SHALL be the single dispatch point: a unified command runs over the session's unified client, built on first use from the configuration and the account the session names (cairn/spec/repl.md), and a protocol-specific command builds its own client and hands it over.

### Requirement: Output goes to stdout
(commands) All data and errors SHALL go to stdout through the printer, with `--json` switching every command to JSON; stderr SHALL carry the logs, the prompts and the welcome only, plus the REPL's help in a JSON session and the failures of the MCP server, whose stdout carries protocol messages only. A command SHALL return a `Serialize + Display` value to the printer rather than printing inline.

## REMOVED Requirements

None.
