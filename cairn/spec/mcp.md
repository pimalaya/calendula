---
cairn: spec
capability: mcp
status: current
---

# MCP server

`calendula mcp` serves the unified API to Model Context Protocol clients over stdio: each tool is a unified command, its input schema generated from the command's grammar and its result the command's `--json` payload, run over a session kept for the server's lifetime (cairn/spec/repl.md). It lists the tools that read by default, the writes under `--allow-write`, and what reaches people beyond the account under `--allow-send` as well. The protocol answers, the call checks, the input grammar and the tool results are pimalaya-cli's shared MCP core (pimalaya-cli cairn/spec/mcp.md); the tool table, the gates and the sending guard are calendula's (src/mcp.rs).

### Requirement: An MCP server over stdio
`calendula mcp` SHALL run an MCP server (revision 2025-06-18) over stdio for one account and one `--backend` on pimalaya-cli's shared MCP core, answering `initialize`, `ping`, `tools/list` and `tools/call`, one JSON-RPC 2.0 message per line, with stdout carrying protocol messages only. A line that is not UTF-8 or not JSON SHALL answer -32700 and one that is not a JSON object, a batch included, -32600, both with a null `id`; a request whose `jsonrpc` is not `"2.0"` SHALL answer -32600 with its `id`; the server SHALL read on after each. Its commands SHALL run over a session kept for the server's lifetime, as the REPL's do, and SHALL never read their input from stdin. A failure of the server itself, resolving its account included, SHALL print as text on stderr and exit 1, leaving stdout empty, a person missing a configuration meeting the welcome instead (cairn/spec/config.md).

#### Scenario: A batch
- GIVEN a server
- WHEN the line `[{"jsonrpc":"2.0","id":1,"method":"ping"}]` is read, then a `ping` with id 4
- THEN the first answers -32600 with `id` `null` and the second `{}`

### Requirement: The server tells the agent how to use it
`initialize` SHALL carry `instructions` saying how to find a calendar (`calendar_list`, the `calendar` argument, the default one when left out) and an event id (`event_list`, its pages and its window, `event_find` by UID), that an event is written from field arguments or iCalendar text and never from a path, that `account_capability` says what the backend serves, that `--allow-write` gates the writes and `--allow-send` the scheduling arguments and any write or delete of an item naming an `ATTENDEE`, and that a failed call answers `isError` with the `calendula-error` object. Every tool name it holds SHALL be one of the tool table.

### Requirement: Tools are the classified unified API
The server SHALL expose one tool per classified leaf of the unified API, named `<group>_<leaf>`: `account_capability`, `calendar_list`, `event_list`, `event_read`, `event_find`, `item_list` and `item_read` read; `calendar_create`, `calendar_update`, `calendar_delete`, `event_create`, `event_update`, `event_delete`, `item_create`, `item_update` and `item_delete` write. `event build` and `item build`, `event agenda` and the protocol-specific commands SHALL NOT be exposed. A unified leaf SHALL be either a tool or explicitly left out, checked by a test.

### Requirement: Tools follow the capabilities
A tool whose verb the account's backend declares `none` (cairn/spec/commands.md, Capability registry) SHALL NOT be listed, as the tables know without connecting: a server over a pimdir store lists no `calendar_create`, `calendar_update` or `calendar_delete`. A tool whose verb is served SHALL stay listed with all its arguments, one the backend does not serve answering `capability-missing` as a tool error.

### Requirement: A tool's input is the command's grammar
A tool's `inputSchema` SHALL be generated from its clap command by pimalaya-cli's `mcp::grammar::Grammar`, each property named as the CLI spells the argument, with `additionalProperties: false`, and a call SHALL be parsed back by that same command from the argv the grammar writes. The composer arguments (`interactive`, `composer`) SHALL be no property of any tool. An argument the `inputSchema` does not declare SHALL be refused before the command runs as a tool error ``Unknown argument `<key>`, <tool> takes <properties>``, and a value of the wrong type the same way; an unlisted tool SHALL be a JSON-RPC error -32602.

### Requirement: A tool's output is the command's payload
A successful call SHALL return the command's JSON value as `structuredContent` and as text, described as `outputSchema` by its registered schema when that schema's root is typed `object`. `event_create`, `event_update`, `item_create` and `item_update`, whose schema is one of two shapes (the write, or an abandoned composer edit), SHALL carry no `outputSchema`. A refused or failed call SHALL return `isError: true` with the `calendula-error` object and no `structuredContent`.

### Requirement: Writing and sending are opt-in
Without flags the server SHALL list reading tools only. `--allow-write` SHALL add the calendar, event and item writes, without the scheduling arguments. `--allow-send` SHALL add the scheduling arguments, `attendee`, `add-attendee`, `remove-attendee`, `organizer` and `send-updates`, and SHALL require `--allow-write`, every write reaching people being a write first.

### Requirement: A write reaches attendees under --allow-send alone
Inside a server without `--allow-send`, a create or an update SHALL be refused when the iCalendar it would write names an `ATTENDEE` in any component, an alarm's included, bytes that do not parse counting as naming one; and an update or a delete SHALL be refused when the stored item names one, read before the write. The refusal SHALL name `--allow-send`, and nothing SHALL be written; a document naming an attendee SHALL be refused before anything is read or connected. A backend, a server or a sync may notify the attendees an item names of its creation, its change or its deletion, which is why the rule holds on every backend, a local store included.

#### Scenario: Cancelling a meeting
- GIVEN a server started with `--allow-write` alone, over a calendar holding an event inviting `bob@example.org`
- WHEN `event_delete` is called on that event
- THEN the call answers `isError` naming `--allow-send`, and the event is still stored

### Requirement: A tool reads no local file
Inside the MCP server, an iCalendar source SHALL be taken as text, a local path naming a file being no way in, and a source given as `-` SHALL be refused with `stdin-taken`.

### Requirement: Tool annotations say what a tool does
A reading tool SHALL be `readOnlyHint`; an update or a delete SHALL be `destructiveHint`. `openWorldHint` SHALL be true exactly when the tool's effect may reach people beyond the account: on `event_create`, `event_update`, `event_delete`, `item_create`, `item_update` and `item_delete` under `--allow-send`, and on no tool otherwise.
