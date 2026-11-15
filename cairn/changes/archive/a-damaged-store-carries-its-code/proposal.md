---
cairn: change
id: a-damaged-store-carries-its-code
status: landed
created: 2026-10-30
---

# A damaged store carries its code

## Why

`store_error` (src/pimdir/client.rs) maps io-pimdir's `PimdirError::code()` to calendula's `ErrorCode` for `store-uncreated`, `store-newer`, `store-foreign`, `store-stale` and `store-busy`, but not for `store-corrupt` (the store's `user_version` and `store_meta.version` disagree) nor `store-hash-algo` (the store names its bodies by a hash this build does not compute), both of which a reader or a producer open raises. cardamum and himalaya map both, so a script met the same store failure with a code from them and none from calendula.

## What

- `ErrorCode::StoreCorrupt` and `ErrorCode::StoreHashAlgo`, mapped from io-pimdir's `store-corrupt` and `store-hash-algo` in `store_error`.
- The JSON code table and the pimdir store failure requirement name both.
