---
cairn: tasks
change: a-write-is-checked-and-queued-in-one-step
---

# Tasks

- [x] Test: a scheduling invitation created where `caldav` supports `calendar.scheduling` in part reports that partial support; fails on the check-then-enqueue order
- [x] `enqueue_with_body` through `PimdirProducer::enqueue_with_body_checked`, the post-write `check` dropped (src/pimdir/backend.rs)
- [x] Lock comments of src/pimdir/client.rs follow io-pimdir's staging window
- [x] `a_store_calendula_cannot_open_carries_its_code` covers a database declaring tables at `user_version` 0 (`store-foreign`)
- [x] Fold the delta into [backends](../../spec/backends.md)
- [x] fmt, clippy, tests (every feature, default, each backend alone); Cargo.lock unchanged
- [x] Log; land
