---
cairn: change
id: the-shared-kit
status: landed
created: 2026-11-12
---

# The shared kit

## Why

Change 4 of the-family-standards. Ortie and Himalaya moved onto pimalaya-cli 0.4 and pimalaya-config 0.4 between 2026-11-10 and 2026-11-12 (ortie cairn/log/2026-11-10-welcome-without-config.md, 2026-11-10-config-flag-repeated.md, 2026-11-11-the-shared-kit.md; himalaya cairn/log/2026-11-11-the-first-run-welcome.md, 2026-11-11-plain-text-output.md, 2026-11-11-wide-lists-print-as-records.md, 2026-11-12-envelopes-print-one-line-each.md), and cardamum the same day (dc2eacd). calendula still rides 0.3: a `-c` split on `:` beside `CALENDULA_CONFIG`, account resolution taking one of several defaults at random, a bare missing-configuration error, and bordered, coloured comfy tables with about 25 colour keys and `-w`. pimalaya-cli 0.4 no longer carries the table module and pimalaya-config 0.4 changes the `TomlConfig` trait, so neither half moves alone.

It also carries the open items of cli-alignment-leftovers, abandoned for it: F1 (`Cli` and `Command`), F7 (the `footer!`), B3 (`caldav.alpn`) and G2 (one `SecretResolver` per account where the unified client assembles its backend).

## What

- Cargo.toml: pimalaya-cli 0.4 (`prompt`, `terminal`; `build` for the build script) and pimalaya-config 0.4 (`toml`, `secret`). pimalaya-cli comes from `[patch.crates-io]` at rev 0fbe5dd597ec48419eac87a9372c0c0d1f40f6f3, the cli master carrying `records::Records` (7a14753), unreleased: the patch waits on the family release publishing pimalaya-cli with `records`, and leaves then. clap takes `default-features = false` with `derive`, `error-context`, `help`, `std`, `suggestions` and `usage`: no `env`, no `wrap_help`, no colour. crossterm leaves, having only typed colours; unicode-width stays for the agenda grid.
- src/cli.rs: `Cli` and `Command`; pimalaya-cli's `footer!`; `-c` as `ConfigFlags`, one path per flag, repeatable, never split, `CALENDULA_CONFIG` gone; the shared commands imported from their modules. `resolve_account` is `Config::load` then `take_account`, returning the account name too. `meet_bare_invocation` prints the help when a configuration exists, a broken one included, and returns `ConfigMissingError` otherwise; `welcome()` builds pimalaya-cli's `Welcome` (bin `calendula`, `event list`, `SAMPLE_RAW_URL`).
- src/main.rs: a `ConfigMissingError` meets a person (`is_wanted`) with the welcome on stderr, exit 1; anything else goes to `ErrorReport` as before.
- src/config.rs: `TomlConfig` gains `sample_url` and `accounts` and loses `take_default_account`. `TableConfig`, `TableArrangementConfig`, every `*ListTableConfig`, `AccountListingConfig`, `CalendarListConfig` and the crossterm colours go, with their getters and merges (src/account/context.rs) and src/shared/table.rs. The top-level `table` and `account` tables and an account's `table` are refused by name (`TableRemovedConfig`, `AccountRemovedConfig`), as `todo` and `journal` are: Himalaya's shape, whose top-level and account blocks deny no unknown key either. A colour key under `event.list.table` or `item.list.table` is refused by its leaf block, which denies unknown keys. `caldav.alpn` joins `gcal.alpn` and `msgraph.alpn`.
- Plain text output: every `Table::new()` site prints through pimalaya-cli's `Columns`, `Fields` or `Records`, every server or user string through `sanitize::text`, every text output ending with one newline; `-w/--max-width` leaves `calendar list`, `event list`, `event find`, `item list` and `pimdir search`; ariadne draws without colour; the agenda grid loses its escapes. `event list` and `event find` print as Himalaya's `envelope list`: a header (`Account: <name> (<backend>)`, `Calendar:`, `UID:` for find), headerless rows (id, start, end, summary last), a footer (`Page:`, `Next page:`). `item list` and `gcal instances` take the same frame (src/shared/listing.rs). The unified client asks a backend for a window one item past the page and pops it, as cardamum's does, so `, the last` and `Next page:` are exact on every backend; `CalendarItemQuery` carries `skip` and `take` in place of `page` and `page_size`, and gcal counts the resources its events fold into to know when the window is covered. The JSON of `event list`, `event find` and `item list` gains `nextPage`.
- G2: `CalendarClient` keeps one `SecretResolver` for its account, so a backend reopened after a composer runs no credential command twice; `CaldavBackend::new` takes the resolver, as the gcal and msgraph backends do.

## Scope / non-goals

The sample's full reshape and its test are the-sample-is-tested-and-shipped's: only its header and its table and colour lines change here. The REPL and the MCP server are the-shared-repl-and-mcp's. `account check` keeps failing on a failed backend rather than printing a partial outcome. The header and footer stay calendula's own (src/shared/listing.rs) until one is lifted into pimalaya-cli.
