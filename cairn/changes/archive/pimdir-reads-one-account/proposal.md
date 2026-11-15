---
cairn: change
id: pimdir-reads-one-account
status: landed
created: 2026-10-30
---

# A pimdir client reads one account

## Why

With `pimdir.account` unset, the pimdir backend read every collection of the store (src/pimdir/backend.rs:451-464), so a store several accounts share showed their calendars merged into one list, and a write staged there carried no account. himalaya derives the account when the store holds one and refuses a store holding several (himalaya/src/pimdir/client.rs:225). The plan of 2026-10-30 settles it for every CLI: one CLI view reads one account; a view across accounts is for the Android and Linux clients. This is item C4 of cli-alignment-leftovers.

## What

- `pimdir.account` unset: the account is the one every collection of the store is grouped under, or none when the store groups none.
- A store whose collections are grouped under several accounts, or partly ungrouped, is refused while `pimdir.account` is unset, naming the accounts and the key.
- The producer is bound to the derived account as it is to a configured one.
- `pimdir status` names the account read, or says the store groups none.

## Breaking

A configuration leaving `pimdir.account` unset over a multi-account store stops working until the key names one account. CHANGELOG states it.
