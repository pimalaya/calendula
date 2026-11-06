---
cairn: tasks
change: pimdir-search
---

# Tasks

## Before

- [x] Decided 2026-11-06: ship on io-pimdir 0c70144 with UTC days; the io-pimdir clock zone, the readable `floating_zone`, neverest's zone and the pimdir vectors moved to pimdir-search-reads-local-days

## This change

- [x] Cargo.toml: `pimdir` enables `io-pimdir/search`; ariadne, optional under `pimdir`
- [x] src/error.rs: `index-missing`, `index-outdated`, `index-version`, `index-zone-unknown`, `index-fts5-missing`, `query-refused`, `query-unanswerable`, with spec/commands.md's code table
- [x] src/pimdir/index.rs: `open_query`, the scope (the kind, `account:`, `collection:`), hits read through the index's reader and rendered with their family's columns (from the summary when bodiless), the coverage with `horizonLeft` and `cut` and its line, the query parse and its refusal (plain and JSON)
- [x] src/pimdir/search.rs: `pimdir search <query>... [--kind event|todo|journal] [-k] [--sort date|name|relevance] [--page] [--page-size]`, a table per kind and JSON (`hits`, `more`, `coverage`), UTC days stated in the help
- [x] src/pimdir/count.rs: `pimdir count <query>... [--kind] [-k]`, JSON (`count`, `coverage`)
- [x] src/pimdir/cli.rs: the two commands beside `status`, `reply` and `cancel`, the family's help updated
- [x] src/json_schema.rs: `calendula-pimdir-search`, `calendula-pimdir-count`
- [x] Tests on a scratch store whose items a test owner drains and whose index the test builds (`PimdirIndex::open`, `refresh`): a word of a `DESCRIPTION` with its snippet; `location:`, `attendee:`; a weekly series begun years ago found by `date:` on a later week; an event at 00:30 UTC found on its UTC day; `--kind todo` with `status:` and `priority:`; a bodiless event matched on its summary and counted in `bodiless`; a range outside the horizon reported `horizonLeft`; `-k` narrowing the hits and the coverage; the configured account alone in a store of two; a queued event not listed until drained and refreshed, `behind` reported in between; every §8.4 refusal code in both renderings; a missing, an outdated and an other-draft index
- [x] Each scope test fails with its scope term removed (kind, account, collection, checked 2026-11-06)
- [x] README pimdir section; CHANGELOG
- [x] Fold delta.md into spec/commands.md, log
