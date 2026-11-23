---
cairn: change
id: follows-storage-draft-17
part: report
---

# Report: calendula follows storage-draft-17

Run on 2026-11-23 on CPUs 6, 7, 22 and 23. Decisions and measures, timed, are in journal.md; the JSON is in bench/.

## Silent losses

None found on calendula's surface. calendula runs no rekey and no drain, so R1, R4 and K4 do not reach it.

## What landed

- The move to io-pimdir 0.20.0: storage-draft-17, sync-draft-15, search-draft-17, git pin 17d4ea8. A store of storage-draft-16 or earlier is refused `store-version`. README gives the start-over steps: let the old engine drain the queue, check `pimdir queue list` is empty, move the store aside, then sync a new one.
- `pimdir` turns on `io-pimdir/calendar`, and CI checks the binary carries calendar, client and search but not indexer (D2). `url` moves to 2.5.9 for RUSTSEC-2026-0592, which predated the move.
- `--sort` takes `newest-first`, `oldest-first`, `title` and `relevance`; `date` and `name` remain as hidden aliases (D1). The search-draft-17 query forms (instants, the `min` to `y` units, `label:`, `property:`, `reserved-form`) are documented. No label or property verbs were added (D3).
- `set-property` rows list in `pimdir queue list`. A malformed row lists beside the valid ones with its decode error, plus an `action-malformed` entry, and the command exits 3 (N1, answered by drop 2).
- Bug B1 fixed: one malformed queue row anywhere in the store failed every `event` listing (D5).
- A windowed listing parses and expands each body once, and `pimdir status` counts without reading summaries.
- A flaky fixture fixed: on draft-17 an index refresh can report `busy` when a parallel test forks while `index.lock` is held, so the fixture retries.
- Tests: 567/567/401/322 for default, --all-features, vdir,pimdir and pimdir (561/561/395/316 before). Clippy clean on all 12 sets.
- J1, cross-app, read-only on the store neverest wrote on drop 3: event list and window show "Quokka review" with its VALARM, `when:` search finds it with complete coverage, `event read` by id and by `--uid` work, status 1/1/0, queue empty. On a copy, `event update` queued an `update` row, listed with its itemId and folded into the listing.

## Tables

Measured under the measure lock on CPU 15, three binary copies, TZ=Europe/Paris, while other lanes were building. Base is 794550c's release binary on storage-draft-15; now is the final binary on storage-draft-17. Both stores come from bench/seed.rs, with 10,000 or 100,000 events plus 200.

| Command (10k unless said) | Base | Now | H4 asks |
|---|---|---|---|
| month window `event list --from 2026-11-01 --to 2026-11-30` | 2.31 s | 1.21 s | 0.5 s |
| `event agenda 11 2026` | 2.33 s | 1.26 s | |
| `event agenda 2026` | 17.4 s | 9.1 s | 3 s |
| month window, 100k | 22.7 s | 11.7 s | 2 s |
| `pimdir status` | 80 ms | 17 ms | ≤ +5% |
| `pimdir search when:2026-11` | 72 ms | 24 ms | ≤ +5% |
| `event search budget` | 70 ms | 8 ms | ≤ +5% |
| `event read 5` | 5 ms | 3 ms | ≤ +5% |
| `event list -s 50` | 12 ms | 12 ms | ≤ +5% |
| `item list -s 50` | 9 ms | 9 ms | ≤ +5% |
| `event update`, warm | 8 to 16 ms | 7 to 9 ms | ≤ +5% |

## Parked

- H3 (D4), reading a window's candidates from the index: built, its equivalence tests failed under two mutations as they should, its output byte-identical, but it missed every threshold (month 1.07 s, year 8.9 s, 100k month 12.2 s). A month window holds 1,112 of the 2,800 series, and about 85% of the time is ical-rs's recurrence walk (`IcalRecurExpand::next`, `day_matches_on`). The patch is kept as bench/H3-parked.patch; its delta sentence is not folded into the spec.
- For the owner: the window thresholds need a faster recurrence walk in ical-rs, not more work in calendula.

## Left

- J1: neverest draining and pushing a calendula row to Radicale (only reads and copy writes were allowed).
- J2: the live suites, waiting on the owner's grant.
- K1: once io-pimdir 0.20.0 is published and the four other CLIs are on it, drop `[patch.crates-io]` (the requirement already says 0.20.0) and run the matrix; calendula's version is the owner's call.
- Out of scope, as the proposal says: CAL-17's queue columns, CAL-9's live cells and the JMAP manual test, the first-run-wizard-frame housekeeping.
- needs-io-pimdir.md N2 (a calendar profile in the generator) and N3 (a missing domain should say so) stay open; the workarounds stand.
