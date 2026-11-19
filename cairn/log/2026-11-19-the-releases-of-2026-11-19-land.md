---
cairn: log
change: the-releases-of-2026-11-19-land
landed: 2026-11-19
---

# The releases of 2026-11-19 land

calendula moved onto ical-rs 0.7.2, io-pimdir 0.18.0 and pimalaya-cli 0.7.0, all from crates.io, and took up what they carry (cairn/changes/archive/the-releases-of-2026-11-19-land).

- **jmap**: ical-rs 0.7.2 with `tzdb`, which calendula already enabled, tells a zoned series' `UNTIL` between UTC and its zone both ways. Test `a_zoned_series_until_round_trips` reads `until` 2026-07-31T09:00:00 Europe/Paris as `UNTIL=20260731T070000Z` and writes it back local; it fails on ical-rs 0.7.1. Ticks jmap-backend's ical-rs task. Two limits remain: a UTC `UNTIL` in a fold's second pass comes back as the first pass, and one of a zone neither the calendar nor the database knows travels unshifted.
- **C5** (pimdir audit-2026-11-12, owner ruling of 2026-11-19): `is_calendar_kind` and the `accounts_of` closure gave way to `PimdirReader::accounts_of`, `calendar_collections` keys on `collection::media_type`, `scope` is `PimdirQuery::scoped`, and the `Incomplete:` counts read `unread_counted()`, `PimdirCalendarCoverage` dropping `reads` and `counts_bodiless`. A collection declaring no kind is no longer a calendar: not listed, not counted toward the account read. Test `a_collection_of_no_kind_is_no_calendar` fails under either old rule; `a_configured_account_reads_an_empty_store` locks what calendula already did.
- **request-origin**: `request_url` raises pimalaya-cli's `error::request_origin`, so `gcal request` and `msgraph request` code the refusal `request-origin` in the family's message. Tests `a_full_url_off_the_api_origin_is_coded_request_origin` and `a_url_off_the_api_origin_sends_nothing`.

Moved: backends.md (pimdir backend), commands.md (pimdir search scope and coverage, the code table, the request output), projection.md (jmap `UNTIL`).
