---
cairn: change
change: pimdir-reads-one-account
---

# Delta

## MODIFIED Requirements

### Requirement: pimdir backend
pimdir SHALL adapt io-pimdir, the store and the sync engine in one crate. The store is an offline cache a sync engine fills, not a server: reads project the store's items and writes are queue actions a later sync applies and propagates.

Collections come from the sync, so `create_calendar`, `update_calendar` and `delete_calendar` SHALL refuse with a message pointing at the account the store syncs.

A collection SHALL be listed as a calendar when it declares `text/calendar`, or when it declares no kind at all (a sync created it before any consumer declared one).

The backend SHALL read one account of the store (pimdir STORAGE §9.2): `pimdir.account` when set, else the one account every collection of the store is grouped under, or none when the store groups none. A store whose collections are grouped under several accounts, or partly ungrouped, SHALL be refused while `pimdir.account` is unset, the error naming the accounts and the key. Reads and writes SHALL be narrowed to that account's collections, and the producer bound to it.
