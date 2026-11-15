---
cairn: change
id: a-check-probes-the-producer
status: landed
created: 2026-10-30
---

# A check probes the producer

## Why

`account check` on pimdir opens a reader and stops there, so a store whose staging lock or write lock a write cannot take passes the check and fails the first `event create`. Since io-pimdir's N8 change (a-producer-proves-it-can-write, 2026-10-30, io-pimdir/cairn/log/2026-10-30-a-producer-proves-it-can-write.md) opening a producer proves nothing either, the lock being taken at the first staged body; io-pimdir now offers `PimdirProducer::probe`, which takes and releases the staging lock without waiting and the database's write lock, writing nothing. cardamum, himalaya and unfog align the same way.

Also moves ical-rs to 99b9273, the revision io-pimdir, himalaya and neverest build, from caa1422, one commit older.

## What

- `check_pimdir` (src/account/check.rs) opens the reader, then a producer, and probes it; a failure carries io-pimdir's code, `store-busy` when a collector holds the staging lock.
- Cargo.toml `[patch.crates-io]` ical-rs at 99b92736a6a076c72936fac29c9b85a5168e4c2f. That commit adds `recur::instant` and `tzdb::version`/`zones` and makes the `tzdb` feature depend on jiff-tzdb directly (already in the lock through jiff), so Cargo.lock gains that dependency line beside the source line. Nothing calendula calls changes.
