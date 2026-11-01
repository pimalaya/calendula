---
cairn: tasks
change: a-body-is-staged-under-the-lock
---

# Tasks

- [x] Pin io-pimdir at 4c3353e in Cargo.toml `[patch.crates-io]`, moving only its lock entry.
- [x] Regression test: a write blocked by a collector holding the staging lock leaves no body in the blob tree; fails on the old order.
- [x] Open the producer first in `create_item` and `update_item`, write the body through `producer.blobs()`, check and enqueue through the same producer.
- [x] Online-meeting check through the reader, no second producer.
- [x] Fix the comments that describe the lock as covering the enqueue alone (src/pimdir/client.rs).
- [x] CHANGELOG Fixed entry.
- [x] Fold the delta into [backends](../../spec/backends.md); log; land.
