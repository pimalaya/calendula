---
cairn: tasks
change: pimdir-follows-draft-10
---

# Tasks

- [x] Cargo.toml and Cargo.lock: io-pimdir 0.11 from crates.io
- [x] `enqueue_with_body`, `PimdirEnqueued` (`handle.id` as the row, `partial` logged), the two `PimdirProducer::check` calls dropped
- [x] `delete_item` states `if_match`; tests: a delete whose body changed underneath is parked, one naming no version applies, a delete or an edit over a conflicted item is parked
- [x] Coverage: `missing`, `unparseable`, `zoneDiffers`, `complete` from `is_complete` alone, the zone line gated on the coverage; tests
- [x] `lock-replaced` coded
- [x] Version tests moved to draft-09 (earlier) and draft-11 (newer)
- [x] Spec folded (backends, commands); CHANGELOG, MIGRATION.md, README; log
