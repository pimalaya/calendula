---
cairn: tasks
change: the-releases-land
---

# Tasks

- [x] Cargo on the released crates, no `[patch]`; build, clippy and the single-backend checks pass.
- [x] jmap on io-jmap's `Calendar/set` and `to_ical`, the local copies removed; `the_calendar_writes_are_calendar_set_calls` and the projection tests pass unchanged.
- [x] C1: `PimdirFormatRefusal`; `a_store_of_another_version_names_the_drain_or_the_upgrade` covers a draft store under `storage-v1`.
- [x] C2: `ItemOrigin`; `a_foreign_body_is_queued_without_its_instructions`, failing when the foreign body is queued whole.
- [x] C4: handles in receipts, `queue show` and `queue cancel`; `a_handle_of_another_store_is_refused_as_a_reset`, `a_queued_reply_prints_its_row_and_the_event_id`.
- [x] CAL-13: `write_draft`; `a_draft_is_private_and_never_written_through_a_planted_path`.
- [x] Fold into backends.md, commands.md, projection.md; CHANGELOG; log.
