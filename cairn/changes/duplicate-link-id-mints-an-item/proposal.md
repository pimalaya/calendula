---
cairn: change
id: duplicate-link-id-mints-an-item
status: landed
created: 2026-08-28
---

# A calendar may hold one UID twice, and the store now says so

> Cross-repo change, same id in eight repositories. This crate is at the end of the chain, and its part is an audit plus a bump: **pimdir** → **io-replica** → **io-pimdir** → **io-webdav** → **neverest** → **himalaya**, **cardamum**, **calendula** (here).

## Why

RFC 4791 §4.1 requires a `UID` to be unique inside its calendar collection. Servers do not always enforce it: verified 2026-08-28 on a Posteo calendar of 454 items, where four `UID`s were held under two hrefs each, both written by Thunderbird. Three pairs differed only in `DTSTAMP` and `LAST-MODIFIED`. The fourth was two genuinely different meetings, "Pre demo woonies" and "Pre demo MINIS", sharing one `UID`.

Until now the store could not represent that: one item per `(collection, link_id)`, so the second resource was frozen and mirrored nowhere. One of those two meetings existed on the server and in no local row. pimdir SPEC §9 changes it: `link_id` becomes the store's key rather than a restatement of the resource's `UID`, bare when free in the collection and minted as `dup:<hint>#<handle>` when the same source already binds it under another handle. Both resources are stored and both list.

Calendula reads that store, and the invariant it never had to state (one item per `UID`) stops holding.

## What

- **Nothing may treat a body's `UID` as an address.** A read resolving a `UID` to the event, or re-deriving one to look a row up, is now wrong: the key stays unique, but it is no longer what the body says. The backend already shows and accepts the public `seq`, so the exposure is limited to whatever resolves an identity directly.
- **The recurrence rule is untouched.** RFC 4791 §4.1 still keeps every component sharing a `UID` in one resource, so a recurring series and its overrides remain one item. What this change admits is a second *resource*, not a second component: two resources, two items, each holding its own whole recurrence set.
- **`create_item` is unchanged, deliberately.** It stages an `add` carrying the derived bare `UID` and returns it, and a staged add colliding with a stored identity still parks (pimdir SPEC §15.3). Minting is what reading a server requires; a locally authored event that collides is a producer error and parks.
- **Two events under one `UID` display as two events.** No badge, no dedup, no merge prompt, and no attempt to reconcile a "Pre demo woonies" against a "Pre demo MINIS": the store reports the multiplicity and resolving it is the user's, against their server.

## Scope / non-goals

- **No repair verb and no merge offer.** Which copy to keep, or whether to re-`UID` one, is a judgement made against the server.
- **No listing change** beyond two rows appearing where one did, in the store's own calendar order.
- **No queue change.** Parking on a colliding local add is the existing rule and stays.
