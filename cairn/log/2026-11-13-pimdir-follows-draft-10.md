---
cairn: log
change: pimdir-follows-draft-10
landed: 2026-11-13
---

# The pimdir backend follows pimdir draft-10

calendula builds against io-pimdir 0.11.0 from crates.io (49e1fd5 plus the version bump; pimdir ae8b231: storage-draft-10, sync-draft-10, search-draft-10), no patch. A store or a search index an earlier draft wrote is refused (`store-version`, `index-version`) through the messages already there.

- **The enqueue.** `enqueue_with_body_checked` is `enqueue_with_body`, and every enqueue answers `PimdirEnqueued`. `delete_item` and the intents (`pimdir reply`, `pimdir cancel`) no longer run `PimdirProducer::check` before they enqueue: they log the partial supports the enqueue answers, one gate per action. The row `pimdir reply` and `pimdir cancel` print stays the integer `handle.id`.
- **A delete names the body it read** (STORAGE §15.3). `delete_item` states as the `remove`'s `if_match` the item's body hash when the delete was gated on a version, none otherwise, and no `resolves`. `a_delete_whose_body_changed_underneath_is_parked` fails without it (the drain applies the remove, `(1, 0)` for `(0, 1)`), checked by putting `if_match: None` back. `a_delete_naming_no_version_removes_whatever_the_item_holds` holds the ungated case, and `a_delete_or_an_edit_over_a_conflicted_item_is_parked` what the owner does with a write calendula queues over a conflict; the sync-landing code of the update test became the helper the four share.
- **Coverage** (SEARCH §8.2). `PimdirCalendarCoverage` gains `missing`, `unparseable` and `zoneDiffers`, flat, summed over the kinds the coverage counts (the query is scoped by `kind:event`). `complete` is `PimdirSearchCoverage::is_complete` alone, which includes the lag. The "Incomplete:" line says a `missing` or an `unparseable` count only while a term of the query reads a calendar body, as the answer is incomplete over one only then. The zone line prints when the coverage reports a differing zone, so for a query holding a date term, where `PimdirCalendarZones::of` compared the two names for every query; `an_index_read_in_another_zone_is_named` now asks `budget date:2026-11` and checks `budget` names none.
- **`lock-replaced`** joins `ErrorCode` and `store_error`.
- **Version tests.** `a_store_calendula_cannot_open_carries_its_code` failed on the move (draft-10, its "newer" store, is calendula's own and opened); it and `a_store_of_another_version_names_the_reinstall_or_the_upgrade` take draft-09 as earlier and draft-11 as newer.
- CHANGELOG, MIGRATION.md and the README follow: draft-10, the coverage members, the zone line, a write parked over a changed or a conflicted event.

Checked: `cargo fmt`, `cargo check --all-features --tests`, clippy `-D warnings` over the eleven feature sets, the unit tests of the binary with every feature (273 pass), tests/account_check.rs and tests/mcp.rs. The suites reaching a provider, and any run against a store neverest wrote, were not run: neverest follows draft-10 first.

Left open, from pimdir cairn/changes/audit-2026-11-12: C1 (the `store-version` wording once the format is `storage-v1`), C2 (`X-PIMDIR-` properties stripped from a resource the user did not write, an owner decision first), C4 (the handle of store id, incarnation and row in receipts), the rest of C5. No command resolves a conflict: a parked write over one is cancelled or retried with the sync engine's own tooling.

Capabilities moved: backends (staged queue actions, the duplicate refusal, the store codes, the body a queued write names, a write settling no conflict), commands (the search coverage, the zone line, the error codes).
