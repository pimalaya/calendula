---
cairn: tasks
change: the-releases-of-2026-11-19-land
---

# Tasks

- [x] Cargo on ical-rs 0.7.2, io-pimdir 0.18.0 and pimalaya-cli 0.7.0 from crates.io, no `[patch]`.
- [x] `a_zoned_series_until_round_trips` (src/jmap/backend.rs), failing on ical-rs 0.7.1.
- [x] C5: `resolve_account` on `accounts_of`, `is_calendar_kind` gone, `calendar_collections` on `media_type`, `scope` on `scoped`, the coverage on `unread_counted`; `a_collection_of_no_kind_is_no_calendar` (failing under either old rule) and `a_configured_account_reads_an_empty_store`.
- [x] `request-origin`: `request_url` raises `error::request_origin`; `a_full_url_off_the_api_origin_is_coded_request_origin` and the gcal command test assert the code.
- [x] Fold into backends.md, commands.md, projection.md; jmap-backend's `UNTIL` task ticked; CHANGELOG; log.
