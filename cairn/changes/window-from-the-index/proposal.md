---
cairn: change
id: window-from-the-index
status: draft
created: 2026-11-26
---

# A pimdir window read from the index

## Why

A pimdir listing over a window walks the whole calendar and expands every series to learn which ones fall in it. At 10,000 events, on storage-draft-17, a month window took 1.21 s, a year agenda 9.1 s, and a month window at 100,000 events 11.7 s (cairn/changes/archive/follows-storage-draft-17/report.md), against the 0.5 s, 3 s and 2 s that change aimed at.

follows-storage-draft-17 built the index path as its task H3 (decision D4) and parked it: the output was byte-identical and its equivalence tests caught two mutations, but it moved the numbers little (month 1.07 s, year 8.9 s, 100k month 12.2 s). A month window holds 1,112 of the 2,800 series, and about 85% of the time is ical-rs's recurrence walk (`IcalRecurExpand::next`, `day_matches_on`), which no candidate set removes (journal.md 18:40 in that archive).

## What

A windowed `list_items` on pimdir takes its candidates from `PimdirIndex` when the index answers the window completely (no lag, the window inside its horizon, no series cut, its floating zone the zone the window is read in, else a margin of a day each way), adds the items the index has not folded, those with no local body (by their summary) and those a pending `update` repoints, and walks the collection otherwise. Either way it lists what the walk lists, the walk staying the oracle in the tests.

The spec sentence, for "A pimdir listing streams the store" in backends.md once this lands, is the one cairn/changes/archive/follows-storage-draft-17/delta.md holds under that requirement.

## Depends on

- A faster recurrence walk in ical-rs: without it the candidate set saves too little to pay for the index path's complexity.
- The parked code, cairn/changes/archive/follows-storage-draft-17/bench/H3-parked.patch, applies on calendula f010f0d (the storage-draft-17 pin) and needs a rebase onto io-pimdir 1.0.0.

## Open question

For the owner: pursue this once ical-rs's walk is faster, or drop it and close this change abandoned.
