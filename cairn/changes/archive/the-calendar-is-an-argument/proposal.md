---
cairn: change
id: the-calendar-is-an-argument
status: landed
created: 2026-11-15
---

# The calendar is an argument

> Line K5 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), finding F22; cardamum's C4 is the twin.

## Why

The collection commands did not take their collection as Himalaya's do (`mailbox delete <MAILBOX>`, `mailbox create <NAME>`): `calendar delete -k <ID>` took a required flag, `calendar update` the `-k` flag, and `calendar create <ID> <NAME>` an id first. `--color` had no short letter where cardamum's `addressbook` commands take `-C`, and `caldav create` and `vdir create` took `-D --description` and `-d --display-name` where `calendar create` takes `-d --description`.

## What

- `calendar create [ID] <NAME>`: the name alone is enough. Given no id, vdir names the directory after the name, refused unless it is one directory name, and CalDAV mints a version 4 UUID for the URL segment, a display name holding what a path cannot (a space, a slash); Google and Graph mint their own either way. `<ID> <NAME>` reads as before.
- `calendar delete <CALENDAR>` and `calendar update [CALENDAR]` take the calendar as an argument, `-k/--calendar` a hidden alias; delete still requires it, update still falls back to `calendar.default`. The MCP tools keep their `calendar` argument.
- `-C/--color` on `calendar create`, `calendar update`, `caldav create` and `vdir create`.
- `caldav create` and `vdir create`: `-n/--display-name`, `-d/--description`, the former `-D` a hidden alias. The former `-d` for the display name cannot stay, `-d` now being the description: MIGRATION.md says so.
- Spec: commands ("Calendar selection", "Protocol-specific families"), backends ("vdir refuses an unknown calendar").
