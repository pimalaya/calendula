---
cairn: log
change: a-body-is-staged-under-the-lock
landed: 2026-10-30
---

# A body is staged under the producer's lock

`create_item` and `update_item` in src/pimdir/backend.rs now open one producer before anything is written, hash the body with it, write the body through `producer.blobs()`, then check and enqueue through the same producer and drop it. The body used to be written through the reader's blob handle before the producer existed, so a collector running in that window deleted it and the owner applied an event with no body (reproduced by the review of 2026-10-30). The online-meeting check reads capabilities through the reader, so a write opens one producer instead of two. A refused write still leaves its body for the collector, since the store's gate reads the new body from the tree; io-pimdir's coming `stage_body` removes that (change pimdir-adopts-the-io-pimdir-release-api).

io-pimdir moves from 8907303 to 4c3353e (docs and deny.toml only); only its source line moved in Cargo.lock, `cargo update -p io-pimdir` having flipped five windows-sys entries that were put back.

Test: `a_write_puts_no_body_in_the_tree_while_a_collector_runs` holds objects.lock exclusively, as `collect_garbage` does, while a create and an update run in another thread, and asserts no file lands in objects/ until it is released; then drains, collects and reads both bodies. On the old order it failed with the created body already in the tree.

Capabilities moved: **backends** (pimdir writes are staged queue actions, modified).
