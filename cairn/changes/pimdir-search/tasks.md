---
cairn: tasks
change: pimdir-search
---

# Tasks

## Before

- [ ] io-pimdir: a zone on `PimdirSearchClock`, `date:` spans and `today`/`yesterday` read in it (SEARCH §8.1), UTC when none; tests on both sides of a UTC midnight
- [ ] io-pimdir: the index's `floating_zone` readable by a query client
- [ ] neverest: `floating_zone` the user's zone (`TZ`, then the system's), in the same move as the io-pimdir clock
- [ ] pimdir: a search vector stating a zone over an index stating the same one, events on both sides of a UTC midnight, one floating and one all-day; SEARCH §8.1 saying a client SHOULD state the user's zone, the one its indexer states
- [ ] The CLIs (neverest, himalaya, cardamum, calendula) on the io-pimdir revision carrying it, or the decision to ship on 0c70144 with UTC dates

## This change

- [ ] Cargo.toml: `pimdir` enables `io-pimdir/search`; ariadne
- [ ] src/error.rs: `index-missing`, `index-outdated`, `index-version`, `index-zone-unknown`, `index-fts5-missing`, `query-refused`, `query-unanswerable`, with spec/commands.md's code table
- [ ] src/pimdir/index.rs: `open_query`, the scope (the kind, `account:`, `collection:`), the clock stating the tables' zone, hits read through the index's reader and rendered with their family's columns (from the summary when bodiless), the coverage with `horizonLeft` and `cut` and its line, the line naming an index read in another zone, the query parse and its refusal (plain and JSON)
- [ ] src/pimdir/search.rs: `pimdir search <query>... [--kind event|todo|journal] [-k] [--sort date|name|relevance] [--page] [--page-size]`, a table per kind and JSON (`hits`, `more`, `coverage`)
- [ ] src/pimdir/count.rs: `pimdir count <query>... [--kind] [-k]`, JSON (`count`, `coverage`)
- [ ] src/pimdir/cli.rs: the two commands beside `status`, `reply` and `cancel`, the family's help updated
- [ ] src/json_schema.rs: `calendula-pimdir-search`, `calendula-pimdir-count`
- [ ] Tests on a scratch store whose items a test owner drains and whose index the test builds (`PimdirIndex::open`, `refresh`): a word of a `DESCRIPTION` with its snippet; `location:`, `attendee:`; a weekly series begun years ago found by `date:` on a later week; an event near midnight found on its local day, the test's index stating the same zone; an all-day event not listed on the next day; an index read in another zone named on the plain output; `--kind todo` with `status:` and `priority:`; a bodiless event matched on its summary and counted in `bodiless`; a range outside the horizon reported `horizonLeft`; `-k` narrowing the hits and the coverage; the configured account alone in a store of two; a queued event not listed until drained and refreshed, `behind` reported in between; every §8.4 refusal code in both renderings; a missing and an outdated index
- [ ] Each scope test fails with its scope term removed; the midnight test fails on a UTC clock
- [ ] README pimdir section; CHANGELOG
- [ ] Fold delta.md into spec/commands.md, log
