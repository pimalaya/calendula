---
cairn: tasks
change: duplicate-link-id-mints-an-item
---

# Tasks

- [x] Bump io-pimdir (and io-replica through it) to the releases carrying the minted key and no ambiguity surface. Nothing in the chain is published yet, so `[patch.crates-io]` points io-pimdir, io-replica and io-webdav at the sibling checkouts instead, which is what makes the whole local suite testable end to end.
- [ ] **Before release**: point those three patch entries back at git or at crates.io. A published calendula cannot carry a path patch.
- [x] Audit src/pimdir/ for a read that assumes one item per identity: none found. Every read resolves a `seq` through `item`, `scan_items` pages on the sort key (the spec already forbids link-id paging here), and `in_range` is per-item. Nothing reads `ambiguous_handles` or `ReplicaStatus::Ambiguous`, so io-pimdir's removal is not breaking here.
- [x] `create_item` needs no change: it stages the derived bare `UID` and returns it for display only, and a collision parks in the store rather than minting.
- [x] The recurrence-set requirement still reads correctly beside the new one: one resource per identity on the wire, two resources possible in one collection.
- [x] Fix the stale test rationale in src/pimdir/backend.rs saying a staged add "deduplicates against" a synced copy. A colliding staged add parks (pimdir SPEC §15.3).
- [x] Tests: two items of one calendar whose bodies carry one `UID` project two items with distinct public ids. Nothing beyond that is testable here, the repo having no store fixture; the parking behaviour is the store owner's and is tested in io-pimdir.
- [x] `cargo test`, `cargo clippy --all-targets`, `cargo fmt`.
- [x] CHANGELOG: folded into the `### Added` entry for the pimdir backend rather than filed under `### Fixed`. The backend has not shipped (v0.1.0 predates it), and `[Unreleased]` is the net diff against the last release, so what a reader needs is the backend's final behaviour, not a fix to a version nobody ran.
- [x] Fold `delta.md` into `cairn/spec/backends.md`; append the log entry; mark `landed`.

## Out of scope, spun out

The audit found one real user-visible loss outside src/pimdir/: the agenda keys its labels by `DTSTART` in a map, so two events starting at the same instant print as one. It predates this change (two unrelated meetings at 09:00 already collide), but this change is what makes it systematically reachable, since a duplicated `UID` now yields two items at the same start. Fixing it changes the `event agenda --output json` shape, so it carries its own change id: `agenda-shows-every-event-at-an-instant`.
