---
cairn: change
id: pimdir-adopts-the-io-pimdir-release-api
status: landed
created: 2026-10-30
---

# The pimdir backend adopts the io-pimdir release API

## Why

The pimdir stable-release plan of 2026-10-30 gives io-pimdir an API the frontends then adopt: `PimdirProducer::stage_body` (hash and write under the staging lock in one call, the reader's blobs read-only), `PimdirError::code()` with a Display that no longer advises "delete the store and let it resync", and a helper refusing a duplicate create. The calendula items that wait on it are gathered here; the rest of the plan's calendula items landed on 2026-10-30 (a-body-is-staged-under-the-lock, pimdir-reads-one-account, a-pimdir-write-says-it-is-queued, pimdir-status-shows-parked-actions, an-online-meeting-ask-is-gated-by-the-item-sources, a-to-do-refusal-names-the-way-out, account-check-fails-on-a-failed-backend, pimdir-docs-cite-storage-and-sync).

Findings behind each (review of 2026-10-30):

- A store error is relayed with io-pimdir's "delete the store and let it resync" (src/pimdir/client.rs, io-pimdir src/client.rs:1327), which destroys queued, unsynced work exactly when versions skew after a release; a script cannot tell a store not created, stale or owned by a newer schema, or busy, from a bug (src/error.rs).
- A create whose `UID` is already live is reported as created, exit 0, then parks at the owner's drain ("link id already present"); `seq_for_link` and the pending creates can refuse it up front.
- calendula writes the body through its own blob writer: a refused write leaves an orphan blob until the collector runs, since the store's gate reads the body from the tree.

## What

- Store errors: map `PimdirError::code()` to calendula's JSON codes, taking io-pimdir's own names for the states a caller acts on (`store-uncreated`, `store-newer`, `store-foreign`, `store-stale`, `store-busy`) rather than one `store-unavailable`, since a newer store asks to upgrade calendula, a foreign database to point calendula at the store, and the others to run its sync engine (an older store is read as it stands since io-pimdir's store version contract, so `store-older` is never raised); the message is io-pimdir's, which says to run or upgrade the sync engine that owns the store, never to delete it.
- Duplicate creates: refuse a create whose link id is live in the calendar or already queued there, with the code `item-duplicate`, naming the live item to update instead or the queued row, through io-pimdir's duplicate check (`enqueue_add`, then `enqueue_with_body`, which runs it on the bytes in hand).
- Staging: write a body and the row pinning it in one step through `PimdirProducer::enqueue_with_body`, which runs the gate on the bytes in hand and, for a create, the duplicate check before the body lands, so a refused write (a duplicate create, an x-vtodo or a scheduling refusal) leaves nothing behind.
- Release: `io-pimdir` requirement to the released version once the user publishes it, the git pins dropped then (the user's call, at the end); CHANGELOG io-pimdir line naming the released version; the pimdir draft citations (`draft-03`, `draft-04`, `ef8eae0`) in CHANGELOG.md updated to the frozen STORAGE and SYNC versions.
- Wizard: a folder holding only a blob tree is no store (src/wizard/local.rs), `pimdir.db` being what a reader opens.
- A missing blob fails `reply` and `cancel`, as it fails `event read`, rather than skipping their checks.

## Blocked on

The release items only: the io-pimdir release carrying `enqueue_with_body`, `PimdirError::code()` and the duplicate refusal (built until then against the local checkout, `path = "../io-pimdir"`), and the pimdir `storage-draft-05` tag.
