---
cairn: log
change: pimdir-docs-cite-storage-and-sync
landed: 2026-10-30
---

# The pimdir docs cite STORAGE

Every "pimdir SPEC n" citation in src/ and cairn/spec/ now names its STORAGE section (§8, §9, §9.2, §15.1, §15.4); none needed SYNC. CONTRIBUTING.md and MIGRATION.md no longer name io-replica (0.2.0 already shipped on io-pimdir 0.5 alone). The `PimdirClient` docs say the producer's lock spans its handle, body write included, and `check_pimdir` says a reader creates nothing.

cairn/spec/testing/pimdir-local.md, the 2026-08-28 test report, left spec/ for [log/2026-08-28-pimdir-local-test-report.md](2026-08-28-pimdir-local-test-report.md), with the personal account it ran against no longer named and a note on what changed since. The link to it in [2026-08-28-pimdir-reader-producer](2026-08-28-pimdir-reader-producer.md), an immutable entry, now points nowhere; this entry is the forward pointer.

Capabilities moved: none (docs only); spec/testing removed.
