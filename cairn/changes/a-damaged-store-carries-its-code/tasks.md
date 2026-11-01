---
cairn: tasks
change: a-damaged-store-carries-its-code
---

# Tasks

- [x] Test: every store failure `store_error` maps carries the code io-pimdir names; fails on the old code for `store-corrupt`.
- [x] `ErrorCode::StoreCorrupt`, `ErrorCode::StoreHashAlgo` (src/error.rs) mapped in `store_error` (src/pimdir/client.rs).
- [x] CHANGELOG: the two codes join the Added entry.
- [x] Fold the delta into [commands](../../spec/commands.md) and [backends](../../spec/backends.md); log; land.
