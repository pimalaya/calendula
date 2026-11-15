---
cairn: tasks
change: the-calendar-is-an-argument
---

# Tasks

- [x] src/shared/calendar/create.rs: `[ID] <NAME>` (`allow_missing_positional`), `-C`; `CalendarClient::create_calendar` and every backend take `Option<&str>`: vdir the name, CalDAV `uuid_v4`, gcal and msgraph warn only on an id given.
- [x] src/shared/calendar/delete.rs, update.rs: positional `CALENDAR`, `required` on delete, the hidden `-k/--calendar` conflicting with it (cardamum's C4 pattern: clap waives the required positional when the flag is present, the usage reads `<CALENDAR>`, the MCP `calendar` stays required); `-C` on update; parse and usage tests in each, tests/mcp.rs reading `calendar_delete`'s required `calendar`.
- [x] src/caldav/create.rs, src/vdir/create.rs: `-n`, `-d` (`-D` a short alias), `-C`.
- [x] Tests failing on the code before: tests/write.rs `the_calendar_commands_take_the_calendar_as_an_argument` (a name alone refused, `calendar delete <ID>` refused); tests/calendar_not_found.rs on the positional forms.
- [x] CHANGELOG, MIGRATION; fold the delta into commands and backends; log.
