---
cairn: tasks
change: pimdir-search-reads-local-days
---

# Tasks

## Before

- [x] io-pimdir: a zone on `PimdirSearchClock`, `date:` spans and `today`/`yesterday` read in it (SEARCH §8.1), UTC when none; tests on both sides of a UTC midnight (io-pimdir 5c18a02, its log 2026-11-06-a-query-states-its-zone)
- [x] io-pimdir: the index's `floating_zone` readable by a query client (`PimdirIndex::floating_zone`, io-pimdir 5c18a02)
- [x] neverest: `floating_zone` the user's zone (`TZ`, then the system's), neverest 7d48121 (change the-index-reads-the-user-s-zone)
- [x] pimdir: a search vector stating a zone over an index stating the same one, events on both sides of a UTC midnight, one floating and one all-day; SEARCH §8.1 saying a client SHOULD state the user's zone, the one its indexer states (pimdir 8c16bb1)
- [x] The CLIs on io-pimdir 5c18a02: neverest 7d48121, himalaya 8cc3f53, cardamum 0a91fde, calendula here

## This change

- [x] src/pimdir/index.rs: the clock stating the tables' zone in `search_items` and `count_items` (`search_clock`, injected by the commands; the tests pass their own)
- [x] src/pimdir/search.rs: the line naming an index read in another zone; the help no longer stating UTC days (count.rs too, the line under the count as well)
- [x] Tests: an event near midnight found on its local day, the test's index stating the same zone; an all-day event not listed on the next day; an index read in another zone named on the plain output; each fails on a UTC clock
- [x] Fold delta.md into spec/commands.md (dropping the UTC sentence), log; CHANGELOG, README
