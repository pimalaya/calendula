---
cairn: tasks
change: the-x14-dependency-pass
---

# Tasks

- [x] X14: Cargo.toml, Cargo.lock; snapshots in src/shared/item/list.rs, src/shared/event/list.rs, src/shared/calendar/list.rs, src/pimdir/queue/list.rs, src/pimdir/search.rs, tests/event_list.rs, tests/pimdir_search.rs, tests/capability.rs; store-version tests on draft-12 and draft-14.
- [x] N31: src/pimdir/queue/show.rs, src/pimdir/status.rs, src/pimdir/backend.rs `status`. Test failing before: pimdir::queue::show `an_applied_row_prints_the_refusals_on_its_event`.
- [x] K26: src/msgraph/backend.rs `create_item`, `update_item`, `change_instances`; src/gcal/backend.rs `create_item`, `update_item`, `change_instances`, `instance`, `project`; src/capability.rs rows. Tests failing before: msgraph::backend::scripted `a_series_creates_its_exceptions`, `a_series_whose_exception_fails_is_deleted_again`, gcal::backend `a_series_creates_its_exceptions`.
- [x] K22: gcal::backend `a_zoned_series_keeps_its_wall_time_across_dst`, failing on io-gcal 0.5.0.
- [x] D28 twin: src/capability.rs `CalendarCreateId`, src/shared/calendar/create.rs. Test failing before: tests/capability.rs `a_calendar_id_on_a_minting_backend_is_refused_before_the_token_is_resolved`.
- [x] Request origins: src/shared/raw_json.rs `request_url`, src/gcal/request.rs, src/msgraph/request.rs. Tests failing before: shared::raw_json `a_full_url_off_the_api_origin_is_refused`, gcal::request `a_url_off_the_api_origin_sends_nothing`.
- [x] CHANGELOG, MIGRATION.md (draft-13); fold the delta; log.
