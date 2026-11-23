---
cairn: log
change: follows-storage-draft-17
date: 2026-11-23
---

# calendula follows storage-draft-17

calendula moves to io-pimdir's storage-draft-17, sync-draft-15 and search-draft-17 revision (git pin cc3021b, drop 1 of the io-pimdir lane, until io-pimdir 0.20.0 is released), with the four other CLIs sharing one store. Plan, decisions and measures: cairn/changes/follows-storage-draft-17 (proposal.md, tasks.md, journal.md, bench/).

- **Dependency**: `[patch.crates-io]` pins io-pimdir; the `pimdir` feature adds `io-pimdir/calendar`, which calendar snippets and `PimdirIndex::item` need; a dev-dependency with `indexer` builds the tests' index. The `features` job of .github/workflows/tests.yml and CONTRIBUTING.md check with `cargo tree -e normal,features` that the binary carries `calendar`, `client` and `search`, and not `indexer`. `url` moves to 2.5.9 (RUSTSEC-2026-0592).
- **Stores**: a store of storage-draft-16 or earlier is refused `store-version`, never migrated; README says how to start over (drain the queue with the old engine, check `pimdir queue list`, move the store aside, sync anew). Guarded by `a_store_calendula_cannot_open_carries_its_code`, its later-engine store now storage-draft-18.
- **commands, `--sort`**: `event search`, `pimdir search` and the MCP tool take `newest-first`, `oldest-first`, `title` and `relevance`; `date` and `name` are hidden aliases. Guarded by `a_search_sorts_by_the_pimdir_order_names` and `the_event_search_tool_lists_the_pimdir_order_names` (tests/pimdir_search.rs).
- **commands, query forms**: instants, the `min`, `h`, `d`, `w`, `mo` and `y` units, `label:`, `property:` and the `reserved-form` refusal, answered by io-pimdir, documented in both helps, the MCP instructions and the README. Guarded by `a_reserved_form_is_refused_and_instants_are_answered` and `initialize_tells_the_agent_how_to_use_the_tools`.
- **commands, queue**: a `set-property` row lists in `pimdir queue list` with its kind and `itemId`. Guarded by `a_set_property_row_of_another_producer_lists`.
- **backends, a queue row that does not decode fails no listing** (ADDED): a listing reads the pending updates of its own calendar, and a calendar whose queue does not decode is read body by body, with a warning. Guarded by `a_malformed_queue_row_fails_no_listing` (tests/pimdir_queue.rs), which failed before the fix.
- **Performance**: a windowed listing parses and expands each body once (a month window over 10,000 events 2.31 s to 1.32 s, a year agenda 17.4 s to 10.1 s, a month over 100,000 events 22.7 s to 12.0 s), guarded by the window tests and `a_streamed_listing_lists_what_the_whole_read_listed`; `pimdir status` counts without summaries (97 ms to 13 ms), guarded by `status_counts_what_a_listing_walks`. Reading a window's candidates from the index (the delta's D4 sentence) is parked: it reached none of its thresholds, the time being the expansion of the series the window holds; the patch and its tests are kept in the change folder.
