---
cairn: log
change: pimdir-typed-summaries
landed: 2026-09-03
---

# The pimdir backend reads typed summaries from the merged io-pimdir

io-replica folded into io-pimdir, and calendula's pimdir backend followed it. The port is mechanical where the types moved (`ReplicaHash` to `object::PimdirHash`, `ReplicaFlags` to `placement::PimdirFlags`, the handles to `client::{reader, producer, blobs}`) and a deletion where the summary changed shape.

**What went** (src/pimdir/backend.rs): `summary_of`, which deserialized the `v: 1` JSON blob into a local `PimdirCalendarMeta`; `stamp_of`, which folded an RFC 3339 or iCalendar stamp onto its leading day; the `derive` wrapper over `conventions::calendar`; the `now` clock the enqueue used to take; and the `meta` field on every staged `add` and `update`. The store's owner derives the summary from the body it finds in the blob tree, so a producer stating one was restating what the drain is about to compute.

**What arrived**: the scan goes through `PimdirReader::list_summaries`, which joins each row's typed summary on the same `(sort_key, seq)` cursor, so an undownloaded item carries a `PimdirSummary::{Event, Task, Journal}` beside its pointer. `is_kind` reads the variant, `in_range` reads `dtstart`, then `due` for a task, through `start_of`. The `PimdirTime` value is verbatim, and `CalendarTimeRange::contains` already compares the leading eight characters, so the stamp folding had nothing left to do.

`create_item` still runs one derivation, `summary::calendar::derive`, for the link id it reports: a queued create has no `seq` yet, and stating the key on the action keeps the reported id and the filed key one value even in a collection whose kind was never declared, where the owner's own derivation would find no convention and park the add.

**Cargo**: `pimdir = ["dep:io-pimdir"]`, io-replica gone, `[patch.crates-io] io-pimdir = { path = "../io-pimdir" }` until the merged crate is released; the requirement stays `0.4`. Cargo.lock moved from io-pimdir 0.4.0 plus io-replica 0.5.0 to the path 0.4.1, which the patch needs (`cargo update -p io-pimdir`) since a patch is not applied over a lock pinning another version.

**Verified**: `cargo build --features pimdir`, the default set and `--no-default-features --features rustls-ring,vdir` all build; `cargo test --all-features` 103 passed, 0 failed (two pimdir tests rewritten over typed rows, one added for the kind answer, one dropped with the stamp folding it tested); `cargo clippy --all-features --all-targets` reports nothing; `cargo fmt --check` clean. No live store was touched: every test builds its rows in memory.

Spec updated: `backends` (MODIFIED "pimdir backend", "The text/calendar summary convention"), `commands` (MODIFIED the kind answer of an undownloaded pimdir item).
