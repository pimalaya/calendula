---
cairn: change
id: the-contract-audit-rulings
status: landed
created: 2026-11-20
---

# The contract audit rulings

## Why

The CLI contract audit of 2026-11-20 (cardamum/calendula report, findings CAL-X1 to X8 and TWN-X1 to X7) found shapes the 1.0 major freezes: capability names in the store's vocabulary, a `pimdir queue show` that is not the `queue list` row, a `caldav delete` deleting a whole calendar when its item id is left empty, an `event search` meaning a UID lookup where cardamum's `card search` runs a query, event rows without `calendarId`, and refusals cardamum codes and calendula did not. The owner ruled on the audit's proposals the same day (A1 to A6, "Go for all").

## What

- `account check` prints pimalaya-cli's list of accounts, its backend rows carrying `tls`; a bare `calendula --json` is refused through pimalaya-cli.
- The TLS tables gain `client.cert` and `client.key`, and the account a `tls` table every network backend inherits field by field (A1).
- `account capability` names the unified commands (`item.delete`, `calendar.update`, `event.add.online-meeting`, `event.update.send-updates`) and gains `event.search`.
- `pimdir queue list`, `queue show` and `status` as cardamum's (A4), with cardamum's `row-not-found`, `confirmation-required` and `page-token-invalid`.
- `caldav delete <CALENDAR> <ITEM-ID>` with the id required, a calendar deleted by `caldav rmcol` (CAL-X8).
- `event read --uid <UID>` reads by UID; `event search <QUERY>` takes the pimdir query language, a pimdir store answering it (A6).
- `calendarId` on every event row, `event build` printing `id` and `calendarId` `null`; wording and text layouts on cardamum's.
