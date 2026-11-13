---
cairn: log
change: pimdir-follows-draft-11
landed: 2026-11-13
---

# The pimdir backend follows pimdir draft-11

calendula builds against io-pimdir cd33f0e (pimdir d8800fb: storage-draft-11, sync-draft-11, search-draft-11), its crate version still 0.11.0 and not released, so pinned by `[patch.crates-io]` until it is. A store or a search index draft-10 wrote is refused (`store-version`, `index-version`) through the messages already there. No production code but the coverage moved: calendula builds no `PimdirSearchReads` by a struct literal, and queues `add`, `update`, `remove` and the two intents alone.

- **Coverage** (SEARCH §8.2). `complete` stays `PimdirSearchCoverage::is_complete`, which now counts a `bodiless` item against an answer only while a term of the query reads a body of its kind. The "Incomplete:" line printed that count under every query, so an event listed and not downloaded would have printed it under a complete answer: it now prints on the terms of `missing` and `unparseable`, read from the coverage (`PimdirSearchReads::kinds`, and `counts_bodiless` for what the crate counts without a term reading the kind, a message under `has:attachment`, which a query scoped by `kind:event` never meets). The three counts stay in the JSON whatever the query reads. A `date:` term reads a calendar body, so a dated query still counts such an event; `organizer:alice@example.org`, a seek of the store, does not.
- **A group read through.** `scope` appends `kind:event`, `account:` and `collection:` to the query's top-level `and`, so a group the user wrote stays a conjunct of the whole and io-pimdir's `conjuncts` now reads through it: `(collection:cal) budget` and `(collection:cal budget)` have the hits and the coverage of `-k cal`. Nothing changed in `scope`.
- **A move over a conflict.** calendula queues no `move` (`gcal move` is the provider's own verb), so nothing it tells the user moved; an `update` or a `remove` over a conflict is parked as the draft-10 follow left it.
- **Version tests.** `a_store_calendula_cannot_open_carries_its_code` failed on the move (draft-11, its "newer" store, is calendula's own and opened); it and `a_store_of_another_version_names_the_reinstall_or_the_upgrade` take draft-10 as earlier and draft-12 as newer.
- **Tests.** `a_body_not_held_counts_against_a_query_reading_it_alone` and `a_bodiless_event_counts_against_a_term_reading_its_body_alone` are new: both fail with the count printed under every query (the line where none was expected), put back by hand, and the first with `counts_bodiless` dropped (no line under `has:attachment` over a message). `a_calendar_narrows_the_hits_and_the_coverage` gained the two grouped queries, which fail on io-pimdir 0.11.0 (`bodiless` 1 and `complete` false where `-k cal` answers 0 and true), the patch taken out for one run. `hits_print_as_records_above_the_coverage_and_zones` and `coverage_renders_as_camel_case_json_and_one_line` state `counts_bodiless`, the first having asserted the count printed under any query.
- CHANGELOG and MIGRATION.md follow: draft-11, an event not downloaded leaving an answer incomplete only under a term reading a calendar body. The README names no draft.

Checked: `cargo fmt --check`, `cargo check --all-features --tests`, clippy `-D warnings` over the eleven feature sets, the unit tests of the binary with every feature (275 pass), tests/account_check.rs, tests/mcp.rs and tests/usage.rs. The suites reaching a provider, and any run against a store neverest wrote, were not run: neverest follows draft-11 first.

Left open, from pimdir cairn/changes/audit-2026-11-12: C1, which this follow leaves as it was (`version_message` orders `storage-draft-N` on both sides: draft-10 and a store naming no version read as an earlier engine's, draft-12 and `storage-v1` as a newer one's; once calendula's own format is `storage-v1` every draft store will read as newer); C2, decided by the owner on 2026-11-13: the raw `item create` keeps honouring the `X-PIMDIR-` properties of an imported file, a departure from pimdir STORAGE §18 the backends spec now states; C4 and the rest of C5. No command resolves a conflict.

Capabilities moved: backends (the departure from §18), commands (the search coverage and its line, a group read through).
