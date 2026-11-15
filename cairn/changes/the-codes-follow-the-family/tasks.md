---
cairn: tasks
change: the-codes-follow-the-family
---

# Tasks

- [x] src/error.rs: `CalendarDefaultMissing` and its helper; the ten pimdir codes.
- [x] src/account/context.rs codes the miss; src/pimdir/backend.rs resolves through it.
- [x] src/pimdir/client.rs: `PIMDIR_CODES` and `ErrorCode::of_pimdir`, `store_error` on them; src/pimdir/backend.rs codes the staging refusals, the performer refusals and the capability reads.
- [x] Tests failing on the code before: tests/write.rs `no_calendar_and_no_default_one_is_coded` (`null`); `a_store_failure_carries_the_code_io_pimdir_names` over the new variants (no code); `a_relayed_pimdir_code_keeps_io_pimdirs_spelling`; `an_online_meeting_in_an_undeclared_store_queues_nothing` reads `unsupported` (`null`).
- [x] CHANGELOG; fold the delta into commands and backends; log.
