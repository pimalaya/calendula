---
cairn: tasks
change: follows-storage-draft-17
---

# Tasks

Cargo runs as `CARGO_TARGET_DIR=~/code/pimalaya/.lanes/calendula/target taskset -c 6,7,22,23 nix develop --command cargo <args> -j 4`. "The matrix" is the twelve-set loop of CONTRIBUTING.md (clippy `-D warnings`, then the tests). Its gate: every set clippy-clean, the test counts of inventory.md "Baseline", plus each test a step adds. Commit unsigned after each gate (frame §4). Write no `//` comment in Rust but `// SAFETY:`.

## 0. Baseline

- [x] 0.1 The matrix at HEAD (794550c expected; re-read the counts if HEAD moved). Gate: inventory.md counts.
- [x] 0.2 Fixtures: build bench/seed.rs under bench/Cargo-v19.toml and bench/Cargo-v20.toml (copy each to a scratch Cargo.toml), then `seed <dir> 10000` and `seed <dir> 100000` per version, on ext4. Gate: `pimdir status` counts n and 200.
- [x] 0.3 Baseline: HEAD's release binary (fat LTO) runs the nine commands of inventory.md "Performance", plus 20 `event update <id> --summary x` on one store copy, under hyperfine on CPU 15 with the measure lock (frame §5). Gate: JSON saved in bench/.

## A. Dependency and build

- [x] A1 Cargo.toml: the frame's `[patch.crates-io]` pin; `pimdir` adds `io-pimdir/calendar`; `[dev-dependencies] io-pimdir = { version = "0.19.1", default-features = false, features = ["client", "indexer"] }`; `cargo update -p io-pimdir`. Gate: `cargo tree --locked -e normal,features -i io-pimdir --no-default-features --features rustls-ring,pimdir` names `calendar`, `client` and `search`, and no `indexer`.
- [x] A2 Put that cargo tree check in the `features` job of .github/workflows/tests.yml and in CONTRIBUTING.md. Gate: the job's script passes locally.
- [x] A3 `nix develop --command cargo-deny check`, `nix flake check` and `nix build .#default`. Gate: all pass and `result/bin/calendula --version` runs.

## B. Compile breaks

- [x] B1 src/pimdir/search.rs:210-229 (`PimdirSearchSortArg`) and src/shared/event/search.rs:142-150, :236-238 (`EventSearchSort`): the four orders of D1, `#[value(alias = "date")]` and `#[value(alias = "name")]`.
- [x] B2 src/pimdir/queue.rs:278-281: `PimdirAction::SetProperty { seq, .. }` joins the `stored(*seq)` arm.
- [x] B3 src/pimdir/index.rs:746 gets `server_object_id: None`; src/pimdir/index.rs:582, :760, :840 and tests/pimdir_search.rs:19, :102 compile under A1's dev-dependency. Gate: `cargo test --no-run` over the matrix.

## C, D. Stamped drafts and SQL

- [x] C1 src/pimdir/backend.rs:3222 `versioned("storage-draft-18")`. Gate: `a_store_calendula_cannot_open_carries_its_code` passes (it fails with storage-draft-16).
- [x] D1 src/pimdir/index.rs:873 `UPDATE summary`. Gate: `a_hit_the_store_cannot_read_is_an_item_error` passes.
- [x] CD Gate: the matrix equals the baseline.

## E. Behaviour inherited

- [x] E1 Tests for D1: `--sort date` gives `newest-first`'s order, `--sort name` `title`'s, `--sort oldest-first` the reverse of the default; the JSON schemas and the MCP tool list the new values.
- [x] E2 The help of `event search` (src/shared/event/search.rs:39-58) and of `pimdir search` (src/pimdir/search.rs:35-60), the MCP instructions (src/mcp.rs:51) and README.md:272-274 document instants (`when:2026-11-23T10:00`), the units `min`, `h`, `d`, `w`, `mo`, `y`, `label:`, `property:` and `reserved-form` (quote `~`, `/`, `{`). Tests: `event search ~x` is refused `query-refused` naming `reserved-form`; `when:2026-11-07T10:00` and `date:2h..` are accepted.
- [x] E3 Test: a `set-property` row another producer queued on a calendar lists in `pimdir queue list` with `kind` `set-property` and its `itemId`.

## G. Bugs

- [x] G1 B1 (D5): `restaged` (src/pimdir/backend.rs:667-678) reads `pending_actions` of the account's calendars, and a calendar that fails to decode makes the scan rule nothing out, with a warning. Test first, failing today: a malformed row in calendar `b` (the src/pimdir/queue.rs:604 patch) leaves `event list`, `event agenda` and `event read --uid` answering on `a` and `b` with exit 0.

## H. Performance

- [x] H1 `perf record` the month window and the year agenda at 10k (the release profile with debug symbols, on a copy); journal.md says where the time goes.
- [x] H2 Parse each body once on the windowed path (`in_range` in src/pimdir/backend.rs:1314-1342, then the projection). Gate: `a_streamed_listing_lists_what_the_whole_read_listed` and the window tests pass, then measure.
- [ ] H3 (parked, journal.md 18:40) D4: a windowed `list_items` takes its candidates from `PimdirIndex` (`occurrences_between`, or a `when:` instants query read with its coverage), adds the items changed since the index cursor, the bodiless ones (by their summary) and the restaged ones, and scans when the coverage is incomplete. Equivalence tests against the scan over `mixed_calendar()` and a 2k seed: index current, behind, past the horizon, floating zone differing.
- [x] H4 Final table (quiet window, measure lock). At 10k: a month window ≤ 0.5 s, a year agenda ≤ 3 s. At 100k: a month window ≤ 2 s. Every other command at most 5% slower than 0.3. Otherwise park H3 with its numbers.

## I. Docs

- [x] I1 Fold delta.md into cairn/spec/commands.md and backends.md, and write the log entry cairn/log/<date>-follows-storage-draft-17.md. CHANGELOG Unreleased: Changed, BREAKING (stores of storage-draft-16 and earlier are refused; the `--sort` values); Fixed (B1). README pimdir section, the start-over steps: let the old neverest drain the queue, check `pimdir queue list` is empty, move the store aside, then sync a new one with the new neverest. Gate: `sh cairn/verify.sh`.

## J. Tests

- [ ] J1 Cross-app (servers lock): neverest at the same pin syncs a local Radicale calendar (neverest tests/pasta.sh) into a fresh store and index; calendula lists, searches, `event update`s and `pimdir reply`s it; neverest drains and pushes; Radicale holds the edit. Record the run in journal.md.
- [ ] J2 The live suites (tests/google.rs, microsoft.rs, fastmail.rs), only with the owner's grant. They touch no pimdir path.

## K. Release

- [ ] K1 Wait until io-pimdir is released (expected 0.20.0, by its tag workflow) and neverest, himalaya, cardamum and carillon are on that same revision: no CLI is released before all five are. Then swap the pin for `io-pimdir = "0.20.0"`, drop `[patch.crates-io]` and run the matrix. calendula's version and tag are the owner's.

## L. Report

- [ ] L1 report.md: what landed, the tables, what is parked.
