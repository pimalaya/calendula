---
cairn: change
change: a-check-probes-the-producer
---

# Delta

## ADDED Requirements

### Requirement: account check takes both pimdir roles
`account check` on pimdir SHALL open the store as a reader, then open a producer and probe it (`PimdirProducer::probe`), which takes and releases the staging lock without waiting and the database's write lock, writing nothing into the store. A store a write cannot reach fails the check rather than the first write, with io-pimdir's code: a collector holding the staging lock fails it at once with `store-busy`, and the check leaves nothing in objects/.

## MODIFIED Requirements

## REMOVED Requirements
