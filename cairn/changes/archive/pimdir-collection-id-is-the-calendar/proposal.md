---
cairn: change
id: pimdir-collection-id-is-the-calendar
status: landed
created: 2026-08-28
---

# A pimdir calendar is its collection id

## Why

The sync engine binds a source's collections under a namespace, so a calendar it caches is keyed `caldav/default` rather than `default`. Calendula already addresses it by that id and shows it whole, which is the right shape.

A collection id is opaque to the store, which neither parses nor validates it (pimdir SPEC 9.2) and models hierarchy through `parent` rather than through a separator, so shortening one would be a guess at the producer's convention rather than a lookup.

Himalaya learnt this the long way, having shipped a derived short name and a `pimdir.namespace` key to rescue the cases the derivation could not decide, and has just removed both.

Calendula never grew either, so the alignment is to say so before someone adds one, and to make the rule usable.

Usable is the gap. A calendar id that names no collection is refused, but the refusal does not say what the store does hold, and an id nobody can guess is one the user has to be shown.

## What

The requirement is stated: a pimdir calendar is its collection id, verbatim, with no derived spelling and no configuration offering one.

The refusal names the calendars the account holds, as Himalaya's does, so the id to type is in the error that asks for it.
