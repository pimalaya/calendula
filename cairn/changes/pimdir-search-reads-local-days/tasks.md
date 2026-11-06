---
cairn: tasks
change: pimdir-search-reads-local-days
---

# Tasks

## Before

- [ ] io-pimdir: a zone on `PimdirSearchClock`, `date:` spans and `today`/`yesterday` read in it (SEARCH §8.1), UTC when none; tests on both sides of a UTC midnight
- [ ] io-pimdir: the index's `floating_zone` readable by a query client
- [ ] neverest: `floating_zone` the user's zone (`TZ`, then the system's), in the same move as the io-pimdir clock
- [ ] pimdir: a search vector stating a zone over an index stating the same one, events on both sides of a UTC midnight, one floating and one all-day; SEARCH §8.1 saying a client SHOULD state the user's zone, the one its indexer states
- [ ] The CLIs (neverest, himalaya, cardamum, calendula) on the io-pimdir revision carrying it

## This change

- [ ] src/pimdir/index.rs: the clock stating the tables' zone in `search_items` and `count_items`
- [ ] src/pimdir/search.rs: the line naming an index read in another zone; the help no longer stating UTC days (count.rs too)
- [ ] Tests: an event near midnight found on its local day, the test's index stating the same zone; an all-day event not listed on the next day; an index read in another zone named on the plain output; the midnight test fails on a UTC clock
- [ ] Fold delta.md into spec/commands.md (dropping the UTC sentence), log; CHANGELOG
