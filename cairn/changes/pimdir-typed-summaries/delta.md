---
cairn: change
change: pimdir-typed-summaries
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: pimdir backend
pimdir SHALL adapt io-pimdir, the store and the sync engine in one crate. The store is an offline cache a sync engine fills, not a server: reads project the store's items and writes are queue actions a later sync applies and propagates.

Collections come from the sync, so `create_calendar`, `update_calendar` and `delete_calendar` SHALL refuse with a message pointing at the account the store syncs.

A collection SHALL be listed as a calendar when it declares `text/calendar`, or when it declares no kind at all (a sync created it before any consumer declared one).

A store grouping its collections under accounts (pimdir SPEC 9.2) SHALL be narrowed to `pimdir.account` when that is set, and read whole when it is not.

### Requirement: The text/calendar summary convention
A pimdir item's summary SHALL be the typed row of the store's event, task or journal table (pimdir STORAGE Annex A.3 to A.5), derived by `io_pimdir::summary::calendar`, the format's own derivation, so an item calendula stages links and summarises exactly as the same item arriving through a sync.

The link id is the bare `UID`, with nothing prepended, and the one derivation calendula runs: a queued create reports it, having no public id yet, and states it on the action so the reported id and the filed key agree by construction.

A queued action carries no summary and no sort key: the store's owner derives both from the body when it applies the action, and a producer deriving them would restate what the owner is about to derive.

calendula SHALL read that row to answer a date question about an item whose body is not local: `dtstart`, then `due` for a to-do carrying no start, the value verbatim so its leading day compares as a parsed `DTSTART` does. It SHALL read the row's table as the item's kind.

### Requirement: A page of a family holds that family
Only its last paragraph moves, the rest of the requirement standing as it is. A pimdir item whose body is not local SHALL answer its kind from the summary table the store filed it in (event, task or journal), which names the component a reader renders the resource as, so narrowing by kind keeps the listing availability-aware.

## REMOVED Requirements

None.
