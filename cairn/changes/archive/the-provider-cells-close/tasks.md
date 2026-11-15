---
cairn: tasks
change: the-provider-cells-close
---

# Tasks

- [x] Tests, each failing on the code before: tests/write.rs `a_local_start_keeps_the_zone_of_the_event` (checked by dropping the new call: the start turned floating); src/pimdir/backend.rs `a_reply_to_an_event_inviting_nobody_queues_nothing` and the codes in `an_event_with_nobody_to_answer_or_notify_queues_nothing`; src/caldav/backend.rs `a_todo_or_a_journal_is_named_after_its_uid`; src/gcal/backend.rs `a_cancelled_event_is_not_found` and `the_primary_calendar_is_the_default`; src/gcal/acl/update.rs `the_scope_is_read_from_the_rule_id` and `a_disagreeing_scope_or_value_is_a_usage_error`; src/msgraph/backend.rs `an_unset_colour_is_none_and_the_default_is_graphs`.
- [x] `inherit_zone` and `start_zone` (src/shared/event/fields.rs), called in src/shared/event/update.rs once the base is read.
- [x] `ErrorCode::InvitationMissing` (src/error.rs), the reply and cancel refusals (src/pimdir/backend.rs).
- [x] `new_resource_name` (src/caldav/backend.rs).
- [x] `GcalBackend::event`, `calendar_from` (src/gcal/backend.rs); `rule_scope` (src/gcal/acl/update.rs).
- [x] Graph `calendar_from` (src/msgraph/backend.rs).
- [x] CHANGELOG (Added, Changed, Fixed).
- [x] Fold the delta into [commands](../../../spec/commands.md) and [backends](../../../spec/backends.md); log.
