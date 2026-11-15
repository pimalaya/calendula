---
cairn: log
change: pimdir-search-pages-by-token
landed: 2026-11-15
---

# pimdir search pages by token

`pimdir search` prints `{hits, nextPage, coverage, errors}` and takes `--page-token`, where it printed `more`; a hit the store cannot read is an `errors` entry and the command exits 3, where the whole search failed; its text prints one line per hit through pimalaya-cli's `Listing`, where it printed records. Line K9 of the CLI alignment round (F12, D9), on the shape Himalaya's c7016650 defined. Capability **commands** moved.

- **src/pimdir/search.rs**: `PageTokenArg`, a page number from 1 winning over `--page`; `PimdirSearchOutput` with `listing`, `nextPage` and `errors`.
- **src/pimdir/index.rs**: `calendar_hit` reads one hit, its failure an item error `Read item <seq>`, coded by the store's refusal else `item-unreadable`; the coverage and the zones note the listing, `pimdir count` keeping its lines.
- **Cargo.toml**: rusqlite as a dev-dependency (bundled, as in Himalaya) for the test corrupting a summary row; the lock adds it to calendula's dependencies alone.
- **Evidence**: `a_hit_the_store_cannot_read_is_an_item_error` failed on the code before, the whole search failing; tests/pimdir_search.rs runs the binary over a filled, indexed store.

CHANGELOG (Changed, breaking), MIGRATION.md. Spec: commands (pimdir search, its coverage and zones, the listings printing `errors`).
