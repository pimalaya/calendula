---
cairn: change
id: pimdir-status-shows-parked-actions
status: landed
created: 2026-10-30
---

# pimdir status shows parked actions

## Why

The owner parks a queued action it cannot apply (a create whose `UID` is already live, a refused capability, a push the server rejects), and the write it carried never lands. `pimdir status` counted pending creates but never read `PimdirReader::parked_actions()` (src/pimdir/backend.rs:412-445): after the duplicate create the review of 2026-10-30 reproduced, status said "1/1 items downloaded" and nothing about the parked row. A write reported as queued that then parks is a silent loss unless something shows it.

## What

- `pimdir status` lists the parked actions of the calendars it reports: the queue row (ROW, the id the store's own operator tooling cancels or retries by), the calendar, the action kind and the error that parked it.
- The JSON output gains `parked: [{"row", "calendarId", "action", "error"}]`, always present.
- A parked action is no item and carries no item id, so the row is the only id shown, under its own name.
