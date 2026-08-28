---
cairn: log
date: 2026-08-16
change: calendar-resource-granularity
---

# The calendar item is the resource, not the component

[pimdir issue 1](https://github.com/pimalaya/pimdir/issues/1) argued that `PRIMARY KEY (collection, link_id)` cannot hold a recurring series and its overrides, since RFC 5545 3.8.4.4 gives an override the same UID as its master, and proposed a composite link id of UID plus RECURRENCE-ID.

The key was not the problem. RFC 4791 4.1 requires the components sharing a UID to live in one calendar object resource, and that UID to be unique within the collection, so at resource granularity the store's key is exactly the uniqueness CalDAV itself enforces. What was missing was any statement of granularity, which is what made the other reading available; pimdir SPEC 13 now opens with it, and this change is calendula holding to it.

## What landed

**[src/pimdir/meta.rs](../../src/pimdir/meta.rs) summarises the master.** It picked `components::<VEVENT>().next()`, the first VEVENT of the resource, and a resource can list an override first, so a series showed as the instance that moved. The pick is now the component carrying no RECURRENCE-ID, falling back to the first when a resource carries overrides alone, which still projects rather than hiding the item.

**The summary carries times verbatim.** `component` replaces `kind`; DTSTART is carried as written, beside the `dtstart_tzid` and the `dtstart_value` that say how to read it; DTEND and DUE the same; `recurring` tells a reader it must expand. The parameters come off the raw content line, since a decoded value drops exactly the two that matter.

The resolved instant is not repeated in the summary. `sort_key` is returned by the store's paging reads and is the single resolved projection under one policy, so a reader never has to decide which of two answers to believe.

**[src/gcal/project.rs](../../src/gcal/project.rs) and [src/gcal/backend.rs](../../src/gcal/backend.rs) fold a recurrence set.** The projection emits RECURRENCE-ID for an event carrying `recurringEventId`, and `set_to_ical` splices the exceptions' components into the master's document under the master's UID. A listing groups on `recurringEventId`; reading one series gathers its exceptions through an iCalUID-filtered listing. The zone minting moved into `define_zones` so it runs again over the folded document, which is what an exception moved into another zone needs.

## The defect the issue did not find

`parse_stamp` reads a stamp on the wall clock and ignores the TZID, which the lens hands over separately, and the summary formatted the result with a trailing `Z`. So `meta.start` asserted a UTC instant while holding local time, for every zoned event: 118 of 128 on the reporter's own calendar. The sort order was self-consistent within one writer, which is why nothing complained, but the field was false and a reader rendering it showed the wrong hour.

The key now resolves through the VTIMEZONE the document carries, taking the earlier offset of a fold and the offset after a gap. Those two hours a year are the reason ical-rs returns `Option<i32>` from `unambiguous()`, and a key has to pick one.

## Where the issue was not followed

**An unresolvable zone is read as floating, not dropped.** The issue proposed the empty key when the document defines no VTIMEZONE for a TZID. On its own reporter's corpus that is 118 items sent to the far end of every listing. A wall-clock reading is wrong by an offset and stays next to where it belongs, so the empty key is kept for a value that does not parse at all.

**The composite link id was declined**, for the reasons in the [proposal](../changes/calendar-resource-granularity/proposal.md): four rows behind one href and one ETag, a push that has to reassemble the resource anyway, and a delete and a revive fanning out over four keys.

**No `recurrence_id` in the summary.** Under resource granularity the item is the whole set, so what a reader needs is `recurring`, not the id of one instance.

## Capabilities moved

- **backends**: the `text/calendar` summary convention now carries verbatim times with their zone and value type, describes the master of a recurrence set, and states the per-component sort key with the policy for each of the four shapes a start may take; the gcal backend's item is a calendar object resource rather than an event.
- **projection**: added the requirement that a recurrence set is one resource, folded on both the listing and the read path, with the zones minted over the folded document.

## Left out

An exception has no independent identity here, which is right for the store and visible in the CLI: a resource carrying a master and an override lists as two rows under one item id, exactly as the CalDAV backend already renders one. Expansion of a recurrence into occurrences stays above the store, since it is a function of when you ask.

pimalaya-linux and pimalaya/android write this same summary under the older names (`kind`, `start`, `tzid`, `all_day`), so both are now out of step with SPEC 13 and need the same pass. The linux writer also carries `until` and `location`, and `until` is a better idea than anything in the issue: it bounds a date-range query over a recurring item without materialising its occurrences. Absorbing it into SPEC 13 is worth a change of its own rather than a silent addition here.
