---
cairn: change
id: pimdir-search
status: landed
created: 2026-11-06
---

# pimdir search through the store's index

## Why

A pimdir store carries a full-text search index (pimdir SEARCH, io-pimdir `PimdirIndex`) with its own query language, built and refreshed by the store's owner (neverest, since its log 2026-10-30-neverest-builds-the-search-index). Himalaya queries it under `himalaya pimdir search` (himalaya cairn/log/2026-10-30-pimdir-search.md). Calendula cannot: `event list`, `todo list` and `journal list` page one calendar and filter by window and kind, so an item is never found by a word of its description, its location or an attendee, by `status:`, `priority:` or `category:`, nor across calendars.

The index also answers dates the way a calendar means them: `date:` compares the occurrences it expanded within its horizon (SEARCH §7, §8.1), so a weekly meeting begun in 2019 is a hit of `date:2026-11-09`, which no store read can answer from the series' `sort_key`.

The query language arrives as a pimdir-specific command first. The unified API, the shared listings taking one query on every backend, is later work (see Later).

## Decisions

1. Two commands, siblings of `pimdir status`, `reply` and `cancel`: `calendula pimdir search <query>...` and `calendula pimdir count <query>...`. No `thread` (SEARCH §9 threads mail alone) and no `tag` (SEARCH §10 tags are placement flags, which no calendar source syncs).
2. Query client only. Calendula opens the index with `PimdirIndex::open_query`, read-only, and never creates, rebuilds nor refreshes it. An answer is the store as of the owner's last refresh: a queued write is not listed nor folded in, and the coverage says how far the index lags.
3. One kind per answer. `--kind event|todo|journal`, `event` by default, adds `kind:event`, `kind:task` or `kind:journal`. Two reasons: the families render one kind per table (commands, "Columns follow the kind", "A page of a family holds that family"), and io-pimdir computes the coverage from the query's top-level terms alone (io-pimdir src/client/search.rs, `coverage`), so the three kinds as one `or` would widen the coverage to every collection of the account, its mail's bodiless count included.
4. Scope. The query is read as its conjuncts plus the kind, `account:<account>` when the store groups the account under a name, and `collection:<id>` when `-k/--calendar` names a calendar. Without `-k` the search spans every calendar of the account: `calendar.default` is not applied, since a search that silently narrows to one calendar reads as "no such event".
5. Order. `--sort date|name|relevance`, `date` by default, `--page` and `--page-size`, as in himalaya. A series sorts by its first occurrence, its `sort_key` (SEARCH §7).
6. A hit. Its item is read through the index's own reader (`index.reader().get_item`, no queue overlay, matching the answer) and rendered with its family's columns (an event its summary, start and end; a todo its summary, due date, status, priority and completion; a journal its summary and date), then the calendar, `matched` (`body`, `summary` or none) and the snippet. Its id is the `seq`, which `event read`, `todo read` and `journal read` take with `-k`. A hit whose body is not local renders from its summary (STORAGE Annex A.3 to A.5); one the store dropped since the refresh keeps the title the index answered.
7. Coverage. Himalaya's fields (`complete`, `bodiless`, `unindexed`, `behind`, `since`, `until`, `filling`) plus the calendar ones io-pimdir already answers: `horizonLeft`, a date range reaching outside the index's horizon (one year back, two forward), and `cut`, the series whose occurrences stop before the range's end. `complete` is false when either holds. The plain output prints one line under the table when the answer is incomplete, and nothing when it is complete.
8. Codes, spelled as himalaya spells them: `index-missing`, `index-outdated`, `index-version`, `index-zone-unknown`, `index-fts5-missing`, `query-refused`, `query-unanswerable`. The missing and outdated messages say the sync engine builds or rebuilds the index, never to delete anything.
9. Refusals. Plain output draws the SEARCH §8.4 span with ariadne, as himalaya does (one new dependency). Under `--json` the error is calendula's error object: the code `query-refused`, with the §8.4 code, span and expectations in `error`, himalaya's deviation, until pimalaya-cli's error report carries structured details.
10. Feature. `pimdir` enables `io-pimdir/search`, with no feature of its own. Calendula already pins ical-rs at 99b9273, the revision io-pimdir's search builds against; mime-rs joins through io-pimdir.
11. Code shape. Himalaya's src/pimdir/index.rs is copied into calendula's src/pimdir/index.rs (scope, hit, coverage, refusal rendering) with src/pimdir/search.rs and count.rs beside it, the shared-client pattern of copying over sharing. With cardamum's pimdir-search, three apps carry the scope builder and the coverage completeness rule: candidates to move down into io-pimdir.

## Depends on

- Shipped on io-pimdir 0c70144 with UTC days: io-pimdir reads every date span in UTC (`PimdirSearchClock` carries `now` alone) and neverest indexes floating times and dates in UTC, so both sides agree and the help and the spec say so. Reading the user's local days, the clock's zone, neverest's `floating_zone` and the line naming an index read in another zone, moved to the change pimdir-search-reads-local-days, which waits for the CLIs' move to the io-pimdir revision carrying the clock zone.
- neverest syncing the account's calendars with its `search` feature (on by default).

## Later

- The occurrences a date range holds, on each event hit. The row shows the item's own start, for a series its first occurrence; listing the occurrences that made it a hit needs the range's instants, which io-pimdir computes in `compile::bounds` but keeps crate-private. Once io-pimdir exposes them, `PimdirIndex::occurrences_between` over that range, kept to the page's hits, gives the rows.
- The agenda from the index: `event agenda` drawing its window from `occurrences_between` and the summaries, as the Android app's agenda does (android cairn/spec/calendar.md), rather than expanding every entry. A change of its own.
- The unified API: the shared listings take a query on every backend, CalDAV, Google Calendar and Graph translating it, pimdir answering it natively, as himalaya plans for its v3.
- io-pimdir's search planner (pimdir cairn/changes/performance-audit-2026-11-05, phase 1) pages by a keyset cursor rather than an offset; calendula moves with himalaya when it lands.
