---
cairn: change
id: pimdir-docs-cite-storage-and-sync
status: landed
created: 2026-10-30
---

# The pimdir docs cite STORAGE and SYNC

## Why

The pimdir specification split its single SPEC into STORAGE, SYNC and SEARCH, versioned apart (plan of 2026-10-30), yet fourteen comments and spec lines still cite "pimdir SPEC n" (src/config.rs, src/pimdir/mod.rs, client.rs, backend.rs, cairn/spec/backends.md, cairn/spec/commands.md), some with a section that moved (blobs are STORAGE §5, not §14). CONTRIBUTING.md and MIGRATION.md still name io-replica, retired and folded into io-pimdir on 2026-09-03. Two comments state wrong facts: opening the store in `account check` "creates the index when absent" (a reader creates nothing), and the producer's lock covers "one enqueue" (it covers the body write too).

cairn/spec/testing/pimdir-local.md is a dated test report of 2026-08-28 living in spec/, which holds current truth only: out of date (it names io-replica and `io_pimdir::conventions`, lists CD2 as open), and run against a personal account's store, which no test may use any more.

## What

- Every "pimdir SPEC n" citation becomes the STORAGE or SYNC section it means.
- io-replica mentions removed from CONTRIBUTING.md and MIGRATION.md.
- The two wrong comments corrected.
- The test report moves to log/2026-08-28-pimdir-local-test-report.md, as history, with the personal account's name dropped and a note that any rerun uses a store synced from a test account (Google pimalaya.org, Microsoft or Fastmail).

Docs only: no behaviour moves.
