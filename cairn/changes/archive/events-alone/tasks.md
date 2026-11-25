---
cairn: tasks
change: events-alone
---

# Tasks

## 0. Verify before cutting

- [x] List what `item` serves that `event` does not, per backend (src/shared/item/, src/capability.rs): anything beyond non-VEVENT components moves into `event` or is named in the proposal before it goes.
- [x] Pin with a test, red first if it fails today: on a vdir calendar holding a VEVENT and a VTODO, `event list` shows the VEVENT alone and `event read|update|delete <vtodo-id>` fail `event-not-found`, the VTODO file byte-identical.
- [x] Check `event read` prints the whole resource iCalendar (overrides, `VTIMEZONE`) as `item read` does, `encoding` included, so nothing `item read` gave is lost.

## 1. Command surface

- [x] Remove `ItemCommand` from src/cli.rs and src/repl.rs, and the CLI, REPL and dispatch modules of src/shared/item/ (`cli.rs`, `list.rs`, `read.rs`, `create.rs`, `update.rs`, `delete.rs`, `build.rs`), keeping the `CalendarItem` types the backends and `event` use.
- [x] Remove the `item_*` MCP tools and their instructions in src/mcp.rs.
- [x] Remove the `calendula-item-*` schemas from src/json_schema.rs.
- [x] Rename the `Item*` capabilities to `Event*` with names `event.*` in src/capability.rs, and the leaves table.
- [x] Rename `ItemNotFound`, `ItemUnreadable` and `ItemInvalid` to `Event*` coded `event-*` in src/error.rs; keep `item-duplicate`.
- [x] Move `item.composer` to `event.composer` in src/config.rs (top level and account), drop `ItemListConfig`; update every reader.
- [x] Reword the help, abouts and the src/main.rs header that name `item`.

## 2. Tests

- [x] Move the item cases of tests/read.rs, tests/write.rs, tests/caldav_id.rs, tests/calendar_not_found.rs, tests/capability.rs, tests/event_list.rs, tests/pimdir_queue.rs, tests/mcp.rs and tests/calendar_duplicate.rs onto `event` where the behaviour they pin still exists, delete them where it does not.
- [x] The sample test (src/config.rs) loads `event.composer`; a sample naming `item.*` fails.
- [x] `cargo fmt`, `cargo test`, and clippy `-D warnings` over the twelve feature sets of the CI `features` job.

## 3. Docs

- [x] config.sample.toml: `event.composer`, no `item.*` key.
- [x] README.md: Features and Usage name no `item`; `calendula` tests/docs.rs passes.
- [x] CHANGELOG.md (the 1.0.0 section): a `**BREAKING**: Removed the item family` entry naming the replacements; the capability, code and configuration renames folded into the entries that introduced them, since none shipped in 0.2.0 under the old names (check each against `git show v0.2.0`).
- [x] Fold delta.md into cairn/spec/commands.md, config.md and mcp.md; log cairn/log/<date>-events-alone.md; status landed, folder to changes/archive/.
- [x] Check the family docs naming calendula's `item`: neverest and carillon READMEs and samples, himalaya-tui, android, the pimalaya.org site.
