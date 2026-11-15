---
cairn: delta
change: pimdir-collection-id-is-the-calendar
---

## ADDED Requirements

### Requirement: A pimdir calendar is its collection id
The pimdir backend SHALL show and accept a calendar as the store's collection id, verbatim: the collection `caldav/default` is the calendar `caldav/default`. It SHALL NOT derive, strip or accept a shortened spelling, and no configuration SHALL offer one.

A sync engine binds a source's collections under a namespace, so an id carries one; the store is opaque to it, neither parsing nor validating an id (pimdir SPEC 9.2) and modelling hierarchy through `parent` rather than through a separator.

Shortening is therefore a guess at the producer's convention, and one that makes a single calendar answer to two spellings.

## MODIFIED Requirements

### Requirement: pimdir refuses an unknown calendar
Every pimdir read and write SHALL fail when the calendar id names no collection of the account.

The store's read seam answers an unknown collection with an empty page and its queue accepts an action for any name, so without this a typo in `-k` would read as an empty calendar and stage into one nothing will ever apply.

The refusal SHALL name the calendars the account holds. Ids carry the sync engine's namespace and are not guessable, so an error asking for one that shows none leaves the user nothing to act on.
