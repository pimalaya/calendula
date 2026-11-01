---
cairn: log
change: pimdir-reads-one-account
landed: 2026-10-30
---

# A pimdir client reads one account

`PimdirClient::new` resolves the account as himalaya does: `pimdir.account` when set, else the one account every collection is grouped under, else none; a store whose collections are grouped under several accounts, or partly ungrouped, is refused, naming them and the key. The producer is bound to the derived account too. `pimdir status` says "none, the store groups no account" when there is none to read. Breaking for a configuration that left the key unset over a multi-account store, which used to read every account's calendars merged. Closes C4 of cli-alignment-leftovers.

Test: `an_unset_account_reads_the_only_one_and_refuses_several`, which on the old code failed with the account read as `None` instead of `work`.

Capabilities moved: **backends** (pimdir backend, modified).
