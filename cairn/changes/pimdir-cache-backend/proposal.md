---
cairn: change
id: pimdir-cache-backend
status: landed
created: 2026-08-08
---

# pimdir cache backend

## Why

calendula could read a remote CalDAV server and a local vdir home, but not a pimdir store: the SQLite-indexed, content-addressed local cache the sync engine (io-replica over io-pimdir) fills. Reading the same store the sync writes gives an indexed, offline, provider-agnostic calendar with no second copy and no format bridge, and it is the same store himalaya and cardamum read for their own domains.

pimdir is a cache, not a live backend: an item may be un- or partially hydrated, and the collections belong to whatever the sync mirrors. The store reports that state; calendula owns the reaction to it.

## What

A `pimdir` feature and `src/pimdir/`, adapting io-pimdir's client read API and the io-replica mutate seam. Reads project the store's items; writes stage mutations a later sync pushes, attributed to the configured source and guarded against a store never synced as it.

The store is kind-agnostic, and `text/calendar` was the one kind whose summary convention the pimdir specification left open. calendula is its first writer, so this change fixes it at `v: 1` and contributes the section upstream.

The collection verbs refuse rather than emulate: a cache does not invent calendars its source does not have, and staging a collection create that no sync would carry is worse than saying so.
