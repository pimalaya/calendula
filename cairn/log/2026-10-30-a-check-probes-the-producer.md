---
cairn: log
change: a-check-probes-the-producer
landed: 2026-10-30
---

# A check probes the producer

`check_pimdir` (src/account/check.rs) opens the reader, then a producer through `PimdirClient::producer`, and calls `PimdirProducer::probe` (io-pimdir a-producer-proves-it-can-write, 2026-10-30), mapping its failure through `store_error`. A store whose staging lock a collector holds now fails the check at once with `store-busy`, where the reader-only check passed and the first write met the lock; the probe writes nothing into the store.

ical-rs moves to 99b92736a6a076c72936fac29c9b85a5168e4c2f, the revision io-pimdir, himalaya and neverest build, from caa1422. That commit adds `recur::instant` (occurrences as instants) and `tzdb::version`/`zone`/`zones`, and makes the `tzdb` feature depend on jiff-tzdb directly; Cargo.lock changes the ical-rs source line and gains that dependency line, jiff-tzdb being already locked through jiff. No calendula call changes.

Test: `a_held_staging_lock_fails_the_check_as_busy` holds objects.lock exclusively, as a collector does, and checks the check fails with `store-busy`, passes once the lock goes, and leaves objects/ empty. On the old check: `panicked at src/account/check.rs:351:48` (`unwrap_err` on `Ok`, the check passed under the lock).

Capabilities moved: **backends** (account check takes both pimdir roles, added).
