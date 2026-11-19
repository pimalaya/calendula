---
cairn: change
id: the-releases-of-2026-11-19-land
status: landed
created: 2026-11-19
---

# The releases of 2026-11-19 land

## Why

io-pimdir 0.18.0, pimalaya-cli 0.7.0 and ical-rs 0.7.2 were released on 2026-11-19, carrying what calendula restated or waited on:

- **ical-rs 0.7.2** tells a zoned `RRULE` `UNTIL` between UTC and the series' zone behind its `tzdb` feature, the upstream task of jmap-backend: over JMAP a zoned series read or written with a UTC `UNTIL` gained or lost its last instance.
- **io-pimdir 0.18.0** carries the helpers of pimdir audit-2026-11-12 C5 (io-pimdir cairn/changes/the-store-rules-the-clis-restate): `collection::media_type`, `PimdirReader::accounts_of`, `PimdirQuery::scoped` and `PimdirSearchCoverage::unread_counted`, which calendula wrote itself (`is_calendar_kind`, the `accounts_of` closure of `resolve_account`, `PimdirClient::scope`, the `reads` and `counts_bodiless` of `PimdirCalendarCoverage`). The owner ruled on 2026-11-19 that a configured account is taken on a store holding no collection yet and that an empty `kind` belongs to no application's kind.
- **pimalaya-cli 0.7.0** codes the refusal of a raw request off its API's origin `request-origin` (`error::request_origin`, cli cairn/log/2026-11-19-a-request-off-its-api-is-coded), which calendula's `gcal request` and `msgraph request` raised uncoded.

## What

- Cargo: ical-rs 0.7.2 (`tzdb` already on), io-pimdir 0.18.0, pimalaya-cli 0.7.0, from crates.io, no `[patch]`.
- jmap: nothing to change in code; a scripted test locks the round trip of a zoned series' `UNTIL`.
- pimdir: `is_calendar_kind` removed; `resolve_account` reads `PimdirReader::accounts_of`, `None` for the configured check and `text/calendar` for the default; `calendar_collections` keys on `collection::media_type`, a collection of no kind no longer listed; `scope` is `PimdirQuery::scoped(Some(Event), account, calendar)`; the coverage line reads `unread_counted()`.
- `request_url` (src/shared/raw_json.rs) raises `error::request_origin`, losing its `provider` argument.
