---
cairn: log
change: the-calendar-is-an-argument
landed: 2026-11-15
---

# The calendar is an argument

`calendar create` takes a name alone, `calendar delete` and `calendar update` take the calendar as an argument, `-k` a hidden alias, and the color is `-C` everywhere, line K5 of the CLI alignment round. Capabilities **commands** and **backends** moved.

- **src/shared/calendar/create.rs, src/shared/client.rs and each backend's `create_calendar`**: `[ID] <NAME>`; with no id vdir takes the name (refused unless one directory name), CalDAV a fresh UUID, the name possibly holding what a URL path cannot; gcal and msgraph warn only on an id given.
- **src/shared/calendar/delete.rs, update.rs**: the positional `CALENDAR`, `required` on delete with the hidden `-k/--calendar` conflicting with it, as cardamum's C4: the usage reads `<CALENDAR>`, both together are refused `usage-invalid`, and the MCP argument stays `calendar`, required on `calendar_delete`.
- **src/caldav/create.rs, src/vdir/create.rs**: `-n/--display-name`, `-d/--description` (`-D` hidden), `-C/--color`; the former `-d` of the display name now sets the description.
- **Evidence**: tests/write.rs `the_calendar_commands_take_the_calendar_as_an_argument` failed on the binary before (`calendar create Work` refused, a missing `<NAME>`).

CHANGELOG (Changed), MIGRATION.md. Spec: commands ("Calendar selection", "Protocol-specific families"), backends ("vdir refuses an unknown calendar").
