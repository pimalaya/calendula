---
cairn: tasks
change: published-dependencies
---

# Tasks

- [x] Check each pinned revision is inside the release it is replaced by
- [x] Cargo.toml: io-gcal 0.3.1, io-msgraph 0.7.1, io-pimdir 0.10, pimalaya-cli 0.3.2, `[patch.crates-io]` gone; Cargo.lock moves those four alone
- [x] `cargo fmt --check`, `cargo clippy --all-targets -D warnings` (default and all features), `cargo test --all-features`
- [x] CHANGELOG: the stale pin clauses dropped, the two fixes the releases bring
- [x] Land pimdir-adopts-the-io-pimdir-release-api; tick cli-alignment-leftovers A2
- [x] Log; land
