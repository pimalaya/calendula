---
cairn: change
id: the-shared-repl-and-mcp
status: landed
created: 2026-11-12
---

# The shared REPL and MCP server

## Why

Change 10 of the-family-standards (owner, 2026-11-12: last before the release polish, so its tests run on the final outputs). Himalaya, Ortie, cardamum and comodoro run a session over one account and a Model Context Protocol server on pimalaya-cli's shared core (himalaya the-shared-repl-and-mcp, landed 2026-11-11; cardamum the same, landed 2026-11-12); calendula has neither. The owner's family rules of 2026-11-12 bind both: the REPL runs the full command tree and refuses what cannot finish inside a session by name; the MCP server lists tools through `Grammar`, gates writes behind `--allow-write` and writes reaching people beyond the user behind `--allow-send` as well, and lists a tool only when the account's capabilities serve it.

## What

- src/session.rs: a `Session` carries the account the global flags name and keeps the unified `CalendarClient` across commands, built on first use and dropped when a command fails, as cardamum's. Every command runs over it (`Command::execute(printer, session)`); the unified commands borrow the kept client (`execute(self, printer, &mut CalendarClient)`), `build` reaches none, the protocol commands build theirs each time.
- `calendula repl`: pimalaya-cli's `Repl` runs the full `Command` tree over the session, `-b`/`--backend` added to the fixed flags, `--json` per line, `#` comments skipped, help on stderr in a JSON session. It refuses by name `repl` and `mcp` (`Cannot start a session inside a session`) and the composer (`-i`, with or without `--composer`) on `event build|create|update` and `item build|create|update`, its editor fighting the session for the terminal it reads. A command reading stdin (an iCalendar source given as `-`) is refused with the new code `stdin-taken` through `pimalaya_cli::session::check_stdin`.
- `calendula mcp`, copied from himalaya/src/mcp.rs and cardamum/src/mcp.rs (`Grammar`, `mcp::serve`, `CapturePrinter`, `ToolResult::printed`, `exit_on_failure`): `account_capability`, `calendar_list`, `event_list`, `event_read`, `event_find`, `item_list` and `item_read` read; `calendar_create|update|delete`, `event_create|update|delete` and `item_create|update|delete` write, under `--allow-write`. `readOnlyHint` on the reads, `destructiveHint` on the updates and deletes. A tool whose verb the account's backend declares `none` is not listed (a pimdir account lists no calendar write). No tool takes the composer flags or reads a local path: an iCalendar source is text, a path naming a file being no way in, as in cardamum. `event build` and `item build` (local formatters, `-o` writing a file) and the protocol commands are not exposed, Himalaya exposing no protocol tool either. Output schemas come from the registry through `tool_output_schema`; the four create and update tools print one of two shapes (an abandoned composer edit being the other) and carry none, as cardamum's.
- The sending gate, below. `--allow-send` requires `--allow-write`: every calendula write that reaches people is a write first, so the flag alone would list nothing.
- An `instructions` paragraph, checked by `the_instructions_name_existing_tools`.

## The sending rule

What reaches people beyond the account in calendula is an item naming attendees (`ATTENDEE`): Google inserts an event the account organizes with `sendUpdates=all`, Graph mails the attendees on a create and an update and a cancellation on a delete, an RFC 6638 CalDAV server schedules implicitly on a `PUT` or a `DELETE` of a scheduling object, and a local store (pimdir, or a vdir that vdirsyncer pushes) hands the write to a sync that reaches such a server. An attendee's own copy deleted may also send the organizer a decline.

- **The scheduling arguments are listed only under `--allow-send`**: `attendee`, `add-attendee`, `remove-attendee`, `organizer` and `send-updates`. `organizer` schedules nothing alone, but it is a scheduling property (RFC 5545 3.8.4.3) meaningful only beside attendees, so nothing is lost leaving it with them.
- **Without `--allow-send`, a write never touches an item naming an attendee**: `CalendarClient` refuses, inside the server, to create or replace an item whose bytes name an `ATTENDEE` (any component, a VALARM's included, bytes that do not parse counting as naming one), and to replace or delete one whose stored copy does, naming `--allow-send`. The document a tool was handed is checked before anything is read or connected; the stored copy costs one read, on an update given a document and on a delete, and only in a server without `--allow-send`. Why refuse rather than strip the attendees or warn: an agent reading an invitation can be told by it to delete or move the meeting, which mails every guest, the same reason Himalaya never sends by default; a refusal naming the flag lets the person decide, and stripping would silently write something else than asked.
- **`openWorldHint` is exact for the server's gates**: false on every tool without `--allow-send`, nothing then reaching beyond the account; true under it on the six event and item writes, which may then notify attendees. The calendar tools stay false.

## Compatibility

New commands and a new error code only: `json-schema` gains nothing (the two commands print no payload, `SILENT` naming them) but `stdin-taken` in the code vocabulary of `calendula-error` and `calendula-account-check`. `event create -`, `event update <id> -`, `item create -` and `item update <id> -` answer `stdin-taken` inside a session; their one-shot behaviour is unchanged.

## Scope / non-goals

The protocol commands keep no client across a session. No tool for `build`, `event agenda` (a cal(1) grid for people) or the protocol commands, `pimdir reply` and `pimdir cancel` included. An argument the backend does not serve (`if-match` on vdir, `online-meeting` off pimdir) stays listed and fails with `capability-missing`, as in cardamum.
