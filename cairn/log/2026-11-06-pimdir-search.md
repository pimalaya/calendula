---
cairn: log
change: pimdir-search
date: 2026-11-06
---

# pimdir search

`calendula pimdir search <query>...` and `calendula pimdir count <query>...` query the store's search index read-only (io-pimdir `PimdirIndex::open_query`, feature `search` now under `pimdir`), on io-pimdir 0c70144 with no version bump. The scope is the query's conjuncts plus `kind:event|task|journal` from `--kind`, `account:` when the store groups the account, and `collection:` from `-k`, all top-level; an unknown `-k` is refused. A hit renders with its family's projection (from the body, else the stored summary, else the index's title; a bodiless event's summary times are projected as a body so they resolve like a listing's), then `calendarId`, `matched` and `snippet`. The coverage adds `horizonLeft` and `cut` to himalaya's fields. The seven index and query codes join `ErrorCode` and `store_error`; refusals are drawn with ariadne (optional dependency under `pimdir`) in plain output and as one line under `--json`.

The proposal recommended waiting for local days; the supervisor decided to ship on UTC days, since io-pimdir 0c70144 reads every date span in UTC and neverest indexes in UTC, so both sides agree. The help and the spec state it. The local-day and other-zone parts moved to the change pimdir-search-reads-local-days, which waits for the CLIs' move to the io-pimdir revision carrying the clock zone.

`PimdirBackend` now keeps the merged `Account` (for page sizes and table colors) rather than `calendar.default` alone; `event list`'s `table_times` and the backend's `known_collection` became `pub(crate)` for reuse.

Tests: 14 new (13 in src/pimdir/index.rs on a scratch store drained by a test owner and indexed at a fixed instant, one rendering test in src/pimdir/search.rs), plus the index and query codes in client.rs's code-mapping test. Removing the `kind:`, `account:` or `collection:` term each failed its scope test (kind: 3 tests, account: 1, collection: 1), checked 2026-11-06 and restored. `cargo test --all-features` 268 passed; clippy clean with all features, each backend alone with rustls-ring, and the default set; `pimdir` alone without `vendored` 220 passed.

Spec: commands (Protocol-specific families, A JSON error carries a stable code, and four added requirements on pimdir search, count, coverage and codes).
