---
cairn: change
id: pimdir-search-reads-local-days
status: active
created: 2026-11-06
---

# pimdir search reads local days

## Why

`calendula pimdir search` and `pimdir count` (change pimdir-search, landed 2026-11-06) read `date:` days, `today` and `yesterday` in UTC, because io-pimdir at 0c70144 reads every date span in UTC and neverest indexes in UTC. In Paris `date:2026-11-07` misses an event at 00:30 on 7 November (23:30 UTC on the 6th) and lists one at 00:30 on the 8th (23:30 UTC on the 7th), and `today` turns over at 01:00. Calendula's tables read times in the local zone (`TZ`, then the system's, commands "Columns follow the kind"), so a search should read days in it too.

## What

1. The search clock states the zone calendula's tables read times in, and `date:` days, `today` and `yesterday` are read in it.
2. When the index reads floating times and dates in another zone (SEARCH §7, `index_meta.floating_zone`), the plain output says so under its table, naming both zones.
3. The `pimdir search` help and the spec drop the UTC statement.

## Depends on

- **io-pimdir, the query's zone.** io-pimdir reads every date span in UTC: `PimdirSearchClock` carries `now` alone and `compile::bounds` cuts days at UTC midnight (io-pimdir src/client/search/compile.rs). SEARCH §8.1 reads a span "in the time zone the client states, UTC when it states none", so the gap is io-pimdir's, not the spec's. io-pimdir needs a zone on `PimdirSearchClock`, the spans and `today` read in it, with vectors in pimdir's vectors/search if the existing ones pin UTC alone.
- **neverest, the index's zone.** The indexer reads floating times and dates in the zone it states (SEARCH §7, `index_meta.floating_zone`), and neverest states none (neverest src/offline/index.rs, `PimdirIndexRefreshOptions` left at its default), so it indexes in UTC: in Paris a floating 09:00 meeting stands at 10:00, and an all-day event on 7 November runs from 01:00 on the 7th to 01:00 on the 8th. neverest needs to state the user's zone (`TZ`, then the system's) as the Android app states the device's; the first refresh in that zone re-expands every calendar item once.
- **Both zones together.** Today both sides read UTC, so the answers are consistently UTC days. Fixing the query's zone alone makes floating and all-day items worse: a search for 8 November in Paris would list the all-day event of the 7th through its last hour. The io-pimdir and neverest changes ship in one move, and the vectors gain a query stating a zone over an index stating the same one.
- **A zone the index does not share.** A store indexed in one zone and searched in another (neverest run under a server's UTC, calendula on a laptop in Paris) is still off for floating and all-day items. io-pimdir keeps the index's `floating_zone` to itself; once a query client can read it, calendula says on the plain output when it differs from the zone the search reads days in.
- That io-pimdir change lands on its current head, storage-draft-09, while calendula, cardamum, himalaya and neverest pin 0c70144 (draft-08) and must read the index version neverest writes. This change waits for the CLIs' move to the io-pimdir revision carrying the clock zone.
