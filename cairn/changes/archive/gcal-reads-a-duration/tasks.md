---
cairn: tasks
change: gcal-reads-a-duration
---

# Tasks

- [x] io-gcal: `GcalEvent::from_ical` computes the end from a DURATION (pimalaya/io-gcal#1).
- [x] Pin io-gcal at `9053761` under `[patch.crates-io]`; `Cargo.lock` holds the one io-gcal.
- [x] `cargo test`, `cargo clippy --all-targets`, with all features.
- [x] Fold delta.md into cairn/spec/projection.md; append the log entry; mark `landed`.
- [x] CHANGELOG: one `### Changed` entry.
- [ ] Live: `event create --duration` on the Google test account.
