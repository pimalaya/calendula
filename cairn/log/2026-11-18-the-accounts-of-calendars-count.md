---
cairn: log
change: the-accounts-of-calendars-count
landed: 2026-11-18
---

# The accounts of calendars count

Landed without a proposal: the twin point agreed with cardamum's session (cardamum 553a3cf), on himalaya's rule of e440b4e4.

With `pimdir.account` unset, `resolve_account` counts only the accounts holding a calendar collection (`text/calendar`, its parameters aside, or no kind, a store synced before kinds were recorded), so a store shared with a mail-only account reads its one account of calendars; several fail `store-account-ambiguous`, ``The pimdir store holds several accounts of calendars (<names>); name one with `pimdir.account` ``. A configured account still checks every account of the store, any kind, and fails `store-account-unknown`. Both codes are new in calendula's table, as in himalaya's and cardamum's.

Tests: `an_account_holding_no_calendar_is_not_counted` (fails on the old count, which named the card account), and `a_configured_account_the_store_lacks_is_refused` now checks the code.

Capabilities moved: backends, commands.
