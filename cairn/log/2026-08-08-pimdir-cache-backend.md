---
cairn: log
change: pimdir-cache-backend
landed: 2026-08-08
---

# pimdir cache backend

Added a `pimdir` backend: calendula over a local pimdir store, the offline cache the sync engine (io-replica over io-pimdir) fills. A `pimdir` feature, `src/pimdir/` (client, hash, meta, backend, status), a `BackendClient::Pimdir` variant selected local-before-network, `Backend::Pimdir`, and a `PimdirConfig { root, source }`.

Reads project the store's items and are availability-aware: an item whose body is not local still lists, carrying no bytes, and only a read of it reports "body not fetched". A range filter still applies to such an item, answered from the stored summary rather than from bytes that are not local, because a cache that hid its own undownloaded items from a date window would answer a different question than the one asked.

Writes go through the io-replica mutate seam, never raw SQL: create stages an `Add`, update an `Edit` (restating the sort key, or an item whose DTSTART moved would stay sorted where its old start put it), delete a `Remove`. Each is attributed to the configured source and guarded: on a store never synced as that source the write fails loudly rather than staging a change no sync will carry. The content hash matches Neverest, himalaya and himalaya-android-m3, so an item calendula adds deduplicates against a synced one.

The collection verbs refuse rather than emulate, naming the account the store syncs. A cache does not invent calendars its source does not have.

Fixed the pimdir `text/calendar` summary convention, which the specification had left open for its first writer: `v: 1` with an optional uid, a required summary, optional RFC 3339 start and end, an optional component kind and an optional size, with the sort key holding DTSTART normalised the same way. The section was contributed upstream to the pimdir specification alongside this change.

Added `pimdir status`, reporting the source writes are attributed to, every source the store has been synced as, and how many of each calendar's items carry a local body. That last number is what turns "body not fetched" from a surprise into something visible in advance.

Spec updated: backends (ADDED the pimdir backend, its public id, its availability-aware reads, its staged and source-guarded writes, its shell-expanded root, its auto-sourced writes, the text/calendar convention; MODIFIED the selection order to place pimdir between vdir and CalDAV), config (ADDED the pimdir block), commands (ADDED the pimdir family).
