---
cairn: change
id: pimdir-follows-storage-draft-09
status: landed
created: 2026-11-06
---

# The pimdir backend follows storage-draft-09

## Why

io-pimdir 5c18a02 (pushed, master, crate 0.9.0) follows pimdir 8c16bb1: storage-draft-09, sync-draft-08 and search-draft-09 (io-pimdir cairn/log/2026-11-05-the-crate-follows-storage-draft-09.md, 2026-11-06-a-query-states-its-zone.md). neverest, which owns the store calendula reads and stages into, moves to the same revision, so calendula has to read the format it writes. The search clock's zone, which pimdir-search-reads-local-days needs, arrives with it.

## What

- `io-pimdir = { version = "0.9", ... }`, the `[patch.crates-io]` rev moved from 0c70144 to 5c18a02. The calendula code compiles unchanged: `PimdirSearchClock` and `PimdirSearchOptions` losing `Copy` touch nothing it does.
- A store draft-08 wrote is refused as `store-version`, never migrated, through the existing earlier-draft message (sync with the neverest that wrote it until nothing is queued, then `neverest sync --reset --force -a <account>`). The wording compares draft numbers against `io_pimdir::sql::FORMAT_VERSION`, so it needs no change; the tests now take draft-08 as the earlier draft and draft-10 as the newer one.
- io-pimdir 0.9 depends on ical-rs 0.7, vcard-rs 0.6 and mime-rs 0.1 from crates.io. calendula's own ical-rs moves from 0.6 to 0.7, compiling unchanged. The ical-rs patch to 99b9273 stays: io-gcal (9053761) and io-msgraph (2ae2637) still require ical-rs 0.6, and dropping it would build them on crates.io 0.6.1, which lacks 99b9273's nesting cap and merge fixes. Two ical-rs versions are linked until calendula moves to io-gcal 0.3 and io-msgraph 0.7.

## Spec

No requirement names a storage draft: the store-version requirement (backends, a pimdir store failure carries io-pimdir's code) reads "an earlier draft's" and holds as written.
