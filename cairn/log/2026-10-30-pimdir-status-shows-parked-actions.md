---
cairn: log
change: pimdir-status-shows-parked-actions
landed: 2026-10-30
---

# pimdir status shows parked actions

`pimdir status` reads `parked_actions()` and lists those targeting the calendars it reports, under "Parked, never applied:" as `ROW <row> <action> in <calendar>: <error>`; its JSON adds `parked: [{"row", "calendarId", "action", "error"}]`. The row is the only id shown, a parked action addressing no item.

Test: `status_shows_the_parked_actions_of_the_account_read` parks a duplicate create in each of two accounts and checks `work` sees its own row alone, with the error. New output, so no old-code run to fail.

Capabilities moved: **commands** (protocol-specific families, modified).
