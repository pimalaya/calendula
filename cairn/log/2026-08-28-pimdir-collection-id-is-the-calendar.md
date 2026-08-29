---
cairn: log
date: 2026-08-28
change: pimdir-collection-id-is-the-calendar
---

# A pimdir calendar is its collection id

Neverest binds a source's collections under a namespace, so the calendar it caches is keyed `caldav/default`. Calendula already addressed it by that whole id, which is the right shape and was never written down.

A collection id is opaque to the store, which neither parses nor validates it (pimdir SPEC 9.2) and models hierarchy through `parent` rather than through a separator, so shortening one is a guess at the producer's convention rather than a lookup.

Himalaya had shipped such a guess, with a `pimdir.namespace` key to rescue the stores it could not decide, and removed both the same day. Stating the rule here is what stops it being added.

## What landed

**The requirement, in cairn/spec/backends.md.** A calendar is its collection id, verbatim, with no derived spelling and no configuration offering one.

**The refusal names what the account holds, in src/pimdir/backend.rs.** `Calendar \`default\` not found` became `Calendar \`default\` not found; this account holds: caldav/default`.

An id carrying a namespace is not guessable, so an error asking for one has to show the choices. `known_collection` already narrowed the store's collections by kind, so the ids it names are calendars and nothing else.

## Capabilities moved

- backends: added *A pimdir calendar is its collection id*; *pimdir refuses an unknown calendar* now names the calendars it holds

## Verification

Built clean and run against the live Neverest store at ~/.local/state/neverest/posteo, which holds mail, calendar and contact collections under one account.

`event list -k default` is refused naming `caldav/default`; `event list -k caldav/default` lists; `calendar list` is unchanged.

86 tests green, clippy clean, `cargo fmt` applied, through the nix devshell.
