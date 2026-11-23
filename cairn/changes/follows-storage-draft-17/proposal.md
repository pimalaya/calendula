---
cairn: change
id: follows-storage-draft-17
status: active
created: 2026-11-23
---

# calendula follows storage-draft-17

## Why

io-pimdir master 7f16b27 implements storage-draft-17, sync-draft-15 and search-draft-17. calendula (0.2.0, on io-pimdir 0.19.1, storage-draft-15) must cross it with the four other CLIs sharing one store, and the owner asked for the bugs and the performance work of its pimdir surface with it.

## Frame

Read /home/soywod/code/pimalaya/pimdir/cairn/changes/the-consumers-follow-draft-17/frame.md first (pin, drafts exact, who edits what, git, CPUs, locks, accounts, decision rules). calendula's share of it:

- CPUs 6,7,22,23; `CARGO_TARGET_DIR` under ~/code/pimalaya/.lanes/calendula/.
- calendula is a reader and a producer: it queues `add`, `update`, `remove`, `calendar-reply` and `calendar-cancel`, and takes the owner role for `pimdir queue cancel` alone. It runs no sync, rekey, drain or index refresh and queues no `move` or `copy`, so frame §3's R1, R4 and K4 do not reach it and section F does not apply.
- calendula is the events CLI: it has no task listing (`item list` is the raw one).

## What

- A. The pin; `pimdir = [..., "io-pimdir/search", "io-pimdir/calendar"]` (calendar snippets and `PimdirIndex::item` need the domain); a dev-dependency on io-pimdir with `indexer` for the tests that build an index; Cargo.lock; nix (the git source goes through pimalaya/nix's `importCargoLock { allowBuiltinFetchGit = true; }`, as neverest's and himalaya's pins do) and cargo deny (the pimalaya org is allowed) checked.
- B. Three production compile breaks and six in the tests (inventory.md), then two failing tests (C, D).
- C. One stamped-draft test: its "later engine" store becomes storage-draft-18. The refusal stays io-pimdir's `PimdirFormatRefusal`.
- D. One test SQL: `UPDATE event_summary` becomes `UPDATE summary`. The queue payload patch and both byte patches hold.
- E. `--sort` takes io-pimdir's order names (D1). The help and the MCP instructions document `reserved-form`, instants and the `min`, `h` and `mo` units. `label:` and `property:` are answered by io-pimdir (D3). `set-property` rows list in `pimdir queue list`.
- G. Bug B1: one malformed queue row anywhere in the store fails every `event` listing, uncoded. The other open lines are triaged in inventory.md.
- H. A baseline at 10k and 100k events. The windowed listing then parses each body once and, when the index answers the window completely, reads only the items the index places in it (D4).
- I. Spec fold, log, CHANGELOG (BREAKING: draft-16 and earlier stores refused, `--sort` values), and a README note on starting over.
- J. The offline suite over the 12 CI sets, a cross-app check with neverest on the same pin, and live runs only with the owner's grant.
- K. Release waits on io-pimdir's release and on the four other CLIs moving to that revision.

## Scope

In: everything above. Out, with the reason:

- CAL-17's queue columns (the performer, answer and occurrence of an intent): the row is pimalaya-cli's shared `PimdirQueueRow` (rulings D19, D21), so pimalaya-cli has to change first, and that repository is not this lane's.
- CAL-9's live cells and the JMAP manual server test: they need live accounts and are not on the pimdir surface.
- first-run-wizard-frame (abandoned but not archived): not pimdir; one line of housekeeping for the owner.
- Label or property verbs (D3).
- Closed and re-checked at 794550c, nothing owed: C1, C2, C4, C5, CAL-9's performance halves, CAL-13's composer half, and ical-rs's UNTIL shift. C2 is no longer a departure: since 2026-11-18 an imported body is queued stripped (`enqueue_with_body`) and only a body the user wrote keeps its `X-PIMDIR-` lines (`enqueue_user_body`). draft-17 leaves §18 unchanged.

## Decisions

- D1 `--sort` on `event search`, `pimdir search` and the MCP tool `event_search` takes `newest-first` (the default), `oldest-first`, `title` and `relevance`, io-pimdir's names. `date` and `name` stay as hidden aliases of `newest-first` and `title`, so scripts keep working. `sender` is not offered: for events it answers in `newest-first` order (SEARCH §8.2). Alternative: keep the old names alone.
- D2 The tests get `indexer` through a dev-dependency. That turns `calendar` on in every test build, so a `cargo tree -e normal,features` gate proves the binary itself carries it.
- D3 No label or property verbs now. Queries accept `label:` and `property:` as io-pimdir answers them, and the docs say so. Whether calendula grows such verbs is for the owner to decide, family-wide.
- D4 A window reads the index first when the index answers it completely: no lag, inside the horizon, no series cut, and the index's floating zone equal to the zone the window is read in (else a one-day margin). Otherwise it scans as today. The scan stays the oracle in the tests.
- D5 For B1, pending updates are read per calendar of the account. A calendar whose queue does not decode makes the listing read every body rather than rule items out by summary. It logs a warning and never fails.

## Risks

1. Release order: a calendula on another io-pimdir revision than neverest, himalaya, cardamum and carillon refuses their store, and they refuse its (frame §2).
2. A masked feature: every test build carries `calendar`, so without D2's gate a release could lose its snippets unseen.
3. D4 could answer differently from the scan (floating times, cut series, index lag, queued writes). The equivalence tests and the fallback hold it.
4. A user who moves a draft-15 store aside before the old neverest has pushed calendula's queued writes loses those writes. The start-over note says to drain the queue first.
