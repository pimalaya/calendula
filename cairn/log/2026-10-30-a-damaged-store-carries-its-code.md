---
cairn: log
change: a-damaged-store-carries-its-code
landed: 2026-10-30
---

# A damaged store carries its code

`store_error` (src/pimdir/client.rs) maps io-pimdir's `store-corrupt` and `store-hash-algo` to the new `ErrorCode::StoreCorrupt` and `ErrorCode::StoreHashAlgo`, as cardamum and himalaya do, so a store whose version stamps disagree, or one naming its bodies by an unknown hash, fails under `--json` with its code.

Test: `a_store_failure_carries_the_code_io_pimdir_names` passes each store failure through `store_error` and checks its code is the one io-pimdir names; on the old code it failed with "no code for Pimdir store is corrupt: PRAGMA user_version is 1 but store_meta records 2".

Capabilities moved: **commands** (A JSON error carries a stable code, modified), **backends** (A pimdir store failure carries io-pimdir's code, modified).
