---
cairn: tasks
change: a-check-probes-the-producer
---

# Tasks

- [x] Test: a held staging lock fails the pimdir check with `store-busy` and leaves objects/ empty; the check passes once the lock goes (failed on the reader-only check: it passed under the lock).
- [x] `check_pimdir` probes a producer after opening the reader.
- [x] ical-rs to 99b9273 (Cargo.toml patch, Cargo.lock source and the jiff-tzdb dependency line).
- [x] CHANGELOG `account check` bullet.
- [x] Fold the delta into [backends](../../spec/backends.md); log.
