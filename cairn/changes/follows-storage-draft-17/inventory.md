---
cairn: change
id: follows-storage-draft-17
---

# Inventory: the evidence, measured on 2026-11-23

Trees, all scratch under ~/code/pimalaya/.pimdir-migration/calendula/: `base` is `git archive` of 794550c; `src` is the same with the frame's pin alone (lock moved by cargo); `trial` is `src` with the fixes of tasks.md A1, B1 (the mapping Date to NewestFirst and Name to Title only), B2, B3, C1 and D1. Logs are in logs/ there. Builds ran on CPUs 6,7,22,23 with `-j 4` while other lanes built.

## Baseline (base, io-pimdir 0.19.1)

The 12 CI sets are clippy-clean. Tests passed (ignored): default 561 (20); `--all-features` 561 (20); `rustls-ring` 196; `native-tls,caldav` 263 (2); `rustls-aws,caldav` 263 (2); `rustls-ring,vdir,pimdir` 395; `caldav` 263 (2); `jmap` 218; `gcal` 235; `msgraph` 237 (6); `vdir` 272; `pimdir` 316. The test gap of cairn/log/2026-11-21 (`rustls-ring,vdir`) is closed at 794550c.

## Breaks (src, the pin alone)

Cargo.lock: io-pimdir becomes the git source, and mime-rs, quoted-printable-codec and vcard-rs leave the lock (`search` no longer pulls them). Clippy fails on the four sets carrying `pimdir` (default, `--all-features`, `vdir,pimdir`, `pimdir`); the eight others pass.

| Where | Error | Cause |
|---|---|---|
| src/pimdir/search.rs:225 | E0599 `PimdirSearchSort::Date` | renamed `NewestFirst` |
| src/pimdir/search.rs:226 | E0599 `PimdirSearchSort::Name` | renamed `Title` |
| src/pimdir/queue.rs:262 | E0004 `PimdirAction::SetProperty` not covered | new action |
| src/pimdir/index.rs:582 (test) | E0432 `PimdirIndexRefreshOptions` | behind `indexer` |
| src/pimdir/index.rs:720 (test) | E0063 missing `server_object_id` | new field |
| src/pimdir/index.rs:760, :840 (test) | E0599 `PimdirIndex::open` | behind `indexer` |
| tests/pimdir_search.rs:19, :102 | E0432, E0599 | behind `indexer` |

## Failing tests (trial, before C1 and D1)

- `pimdir::backend::tests::a_store_calendula_cannot_open_carries_its_code` (src/pimdir/backend.rs:3232): storage-draft-16 now reads as an earlier engine's ("move its directory aside"), not "upgrade calendula".
- `pimdir::index::tests::a_hit_the_store_cannot_read_is_an_item_error` (src/pimdir/index.rs:875): "no such table: event_summary".

After C1 and D1, the trial gives the baseline's counts on all 12 sets, clippy-clean (logs/trial-summary.txt).

## SQL and byte patches

- src/pimdir/index.rs:868-874 `UPDATE event_summary`: breaks (D1). The `summary` table keeps `recurring` with its CHECK (io-pimdir spec/migrations/storage/0001_init.sql:537-561).
- src/pimdir/queue.rs:604-606 `UPDATE queue SET payload`: holds.
- src/pimdir/backend.rs:3203-3222 and src/pimdir/index.rs:1339-1351: `FORMAT_VERSION` is still spelled once in pimdir.db and in index.db; both tests pass on the trial.

## Features

- Without `calendar`, `PimdirExtract::text` answers `None` for `text/calendar`, so the calendar hits carry no snippet (io-pimdir src/client/search.rs:2941, src/search/extract.rs:154-163; read from code). `indexer` pulls `calendar`, so test builds never show it.
- On the trial, `cargo tree --locked -e normal,features -i io-pimdir --no-default-features --features rustls-ring,pimdir` names `calendar`, `client` and `search`.
- MSRV: io-pimdir 7f16b27 states rust-version 1.89, as calendula does. deny.toml allows git sources of the pimalaya org.

## What a user meets (trial debug binary)

- A storage-draft-15 store: `{"code":"store-version", ... "Pimdir store is storage-draft-15, written by an earlier sync engine, and calendula reads storage-draft-17 alone: a store is never migrated, so sync it with the engine that wrote it until nothing is queued, then move its directory aside and sync a new store"}`, exit 1; `account check` fails the row the same way.
- A search-draft-15 index in a draft-17 store: `index-version`, saying an earlier index is rebuilt by its sync engine.

## Bug B1

One `add` queued in `bench/holidays`, its payload patched as src/pimdir/queue.rs:605 patches it, then:

- `event list -k bench/calendar` answers `{"code":null,"error":"Pimdir action error: Pimdir action payload states no integer version"}`, exit 1, on both base (0.19.1, stores/bad19) and trial (7f16b27, stores/bad20). `event read --uid` fails the same.
- `item list` on either calendar answers; `pimdir queue list` names `action-malformed`.
- Cause (read from code): every kind or UID filter calls `restaged` (src/pimdir/backend.rs:667-678), which reads the store-wide `list_pending_actions()` with no code mapped.

## Open lines (re-checked at 794550c)

| Line | Source | Verdict |
|---|---|---|
| C1 | pimdir cairn/changes/audit-2026-11-12/tasks.md:102 | closed: src/pimdir/client.rs:230-238 `PimdirFormatRefusal` |
| C2 | same, :103 | closed, no departure: src/pimdir/backend.rs:1041-1042, test `a_foreign_body_is_queued_without_its_instructions` |
| C4 | same, :105 | closed: handles in `queue show`, `cancel` (cairn/log/2026-11-18-the-listed-rows-carry-their-handle.md) |
| C5 | same, :106 | closed for calendula: `accounts_of`, `scoped`, `unread_counted`, `media_type` (src/pimdir/client.rs:113-155, index.rs:149-155, :415, backend.rs:597) |
| CAL-9 performance | cairn/log/2026-11-19-a-page-reads-its-own-bodies.md, -a-uid-search-asks-for-its-uid.md | landed; live cells left, off the pimdir surface |
| CAL-13 composer | cairn/log/2026-11-18-the-releases-land.md | landed |
| CAL-17 columns | cairn/log/2026-11-18-a-performer-syncs-the-calendar.md | open, a pimalaya-cli row change first: out |
| UNTIL shift | cairn/changes/jmap-backend/tasks.md | landed in ical-rs 0.7.2 |
| JMAP manual test | same | open, live: out |
| `pending_actions` all or nothing | ~/.claude/plans/pimalaya-family-ledgers/cli-contract-audit-2026-11-20/handoff-pimdir.md:17 | io-pimdir: needs-io-pimdir.md N1 |
| GitHub issues | `gh issue list -R pimalaya/calendula` | none open |

## Performance (base release binary, no fat LTO, 10k store)

The fixture: bench/seed.rs at 10,000 events plus 200, seeded in 92 s (75 s of it the index). The bodies are 36% timed UTC, 24% Europe/Paris with a VTIMEZONE, 10% all-day, 2% floating and 28% series (weekly with UNTIL, weekly open, daily COUNT, monthly, yearly, Outlook-zoned weekly; 30% of the timed series carry EXDATEs, half of those an override). 25% carry attendees. `TZ=Europe/Paris`, hyperfine, 5 runs, under load:

| Command | Mean |
|---|---|
| `event list -s 50` | 16.3 ms |
| `event list --from 2026-11-01 --to 2026-11-30` (3,870 occurrences) | 2.613 s ± 0.521 |
| `event agenda 11 2026` | 2.495 s ± 0.430 |
| `event agenda 2026` | 13.342 s ± 0.034 |
| `pimdir search when:2026-11` | 59.4 ms |
| `event search budget` | 59.5 ms |
| `event read 5` | 4.2 ms |
| `item list -s 50` | 7.8 ms |
| `pimdir status` | 64.7 ms |

The pimdir generator's count profile files events with no recurrence (io-pimdir spec/generator/src/profile.rs:1165-1182), hence the seed tool.
