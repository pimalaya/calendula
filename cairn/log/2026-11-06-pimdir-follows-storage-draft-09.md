---
cairn: log
change: pimdir-follows-storage-draft-09
landed: 2026-11-06
---

# The pimdir backend follows storage-draft-09

calendula builds against io-pimdir 5c18a02 (crate 0.9.0; pimdir 8c16bb1: storage-draft-09, sync-draft-08, search-draft-09): `io-pimdir = "0.9"`, the `[patch.crates-io]` rev moved from 0c70144. No calendula code changed for it.

- Cargo.lock moves io-pimdir (0.8.0 to 0.9.0) and its dependencies alone: ical-rs 0.7.0, mime-rs 0.1.0, quoted-printable-codec 0.1.0 and vcard-rs 0.6.0 from crates.io replace the git revisions, hashlink 0.12.2 joins through rusqlite's statement cache, and the git revisions of base64-codec 0.0.1 and simd-cpu 0.0.1 leave, their crates.io 0.1.0 already in the lock.
- calendula's own ical-rs moves to 0.7 from crates.io, sharing io-pimdir's, with no source change. The ical-rs patch to 99b9273 stays for io-gcal 9053761 and io-msgraph 2ae2637, which require 0.6: without it they built on crates.io 0.6.1, which lacks 99b9273's nesting cap of 16 and its merge fixes. Two ical-rs versions are linked until those move (io-gcal 0.3 and io-msgraph 0.7 require 0.7).
- A store draft-08 wrote is refused as `store-version` with the earlier-draft message; `version_message` compares draft numbers to `io_pimdir::sql::FORMAT_VERSION`, so the wording follows unchanged. The tests `a_store_of_another_version_names_the_reinstall_or_the_upgrade` and `a_store_calendula_cannot_open_carries_its_code` failed on the move (draft-09, their "newer" store, is now calendula's own and opened) and now take draft-08 as earlier and draft-10 as newer; `a_missing_or_outdated_index_says_to_run_the_sync_engine` spelled search-draft-08 and now reads `sql::search::FORMAT_VERSION`.
- MIGRATION.md and the CHANGELOG name storage-draft-09.

Capabilities moved: none (no requirement names a draft).
