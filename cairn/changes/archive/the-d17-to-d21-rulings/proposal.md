---
cairn: change
id: the-d17-to-d21-rulings
status: landed
created: 2026-11-15
---

# The D17 to D21 rulings

> Rulings D17 to D21 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15/tasks.md), task X8, as pimalaya-cli's spec states them at 595504e (cairn/spec/error.md, *A code names the domain's noun*; cairn/spec/output.md, *Paging resumes*, *Protocol commands are raw-faithful*, *A queue row keeps one shape*); cardamum is the twin.

## Why

X3, the round's last check, found a calendar created twice succeeding on vdir, writing its markers over the existing ones (row M4); the `msgraph` listings printing Graph's `@odata.nextLink` and taking no flag to resume (M1); and the `pimdir queue list` rows in a shape of calendula's own, the parked ones lacking `linkId`, `itemId` and `summary`, the pending ones `attempts`, `unconfirmed` and `error` (M9). The owner ruled each the same day.

## What

- D17: a create naming a calendar that exists fails `calendar-duplicate` wherever the backend can tell. vdir can (the directory exists: `calendar create` and `vdir create`, where 0.2 wrote over its markers and exited 0); CalDAV can (a MKCALENDAR answered naming `DAV:resource-must-be-null`, or 405 or 412 with the calendar then listed); Graph can (`ErrorFolderExists`, Exchange keeping one calendar per name). Google Calendar mints the id and lets two calendars share a name: nothing to refuse. pimdir serves no calendar create.
- D18: `msgraph calendars list`, `events list` and `events delta` print `nextPage`, Graph's `@odata.nextLink` lifted out of the page, and take `--page-token` to resume, the link carrying the calendar and the query, which are refused beside it. The delta's `@odata.deltaLink` stays raw, `--delta-link` taking it back for the next round. The `gcal` listings read every page and print no cursor.
- D19 with D21: every `pimdir queue list` row, pending or parked, prints `{row, kind, collection, linkId, itemId, producer, queuedAt, attempts, unconfirmed, error, summary}`, every key, `kind` as io-pimdir holds it.

D17 breaks a script that created a calendar over an existing one: CHANGELOG and MIGRATION.md say so. The `msgraph` namespace and `pimdir queue` are new since 0.2, so D18 and D19 break no 0.2 user: their [Unreleased] lines are reworded.
