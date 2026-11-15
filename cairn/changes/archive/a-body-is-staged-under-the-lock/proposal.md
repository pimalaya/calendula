---
cairn: change
id: a-body-is-staged-under-the-lock
status: landed
created: 2026-10-30
---

# A body is staged under the producer's lock

## Why

pimdir STORAGE §8 requires a producer to hold the staging lock on objects.lock across the blob write and the enqueue that pins it. The pimdir backend wrote a new or edited body through the reader's blob handle (src/pimdir/backend.rs:593-606) and only then opened the producer that takes the lock (src/pimdir/backend.rs:617). A collector running in between found an unreferenced file and deleted it; the owner's drain still applied the `add`, and the event lost its body ("Body blob missing"). The review of 2026-10-30 reproduced it on a scratch store, for `create_item` and `update_item` alike.

`create_item` and `update_item` also opened two producers per write, one for the online-meeting check and one for the enqueue.

cardamum does it right (cardamum/src/pimdir/backend.rs:416-447): one producer opened first, wrapping the body write and the enqueue.

## What

- One producer per write, opened before anything is written: the body is hashed with the producer's hash and written through `producer.blobs()`, then the action is checked and enqueued through the same producer, which is dropped after.
- The online-meeting check reads capabilities through the reader, so a write opens no second producer.
- io-pimdir pinned at 4c3353e, the revision the other pimdir frontends move to (8907303..4c3353e is docs and deny.toml only).

## Out

- io-pimdir's `PimdirProducer::stage_body`, which hashes and writes under the lock in one call and keeps the reader's blobs read-only: change pimdir-adopts-the-io-pimdir-release-api.
- A refused write still leaves its body in the blob tree for the collector: the store's gate reads the new body from the tree, so the write has to come first until `stage_body` exists.
