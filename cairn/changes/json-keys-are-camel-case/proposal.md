---
cairn: change
id: json-keys-are-camel-case
status: landed
created: 2026-08-29
---

# The `--json` keys read as the wire formats they wrap

The group B pass settled every payload key on kebab-case, reasoning from the configuration schema. That was the wrong neighbour to align on.

A configuration is a TOML document a human writes, and kebab-case is the family's TOML vocabulary. A `--json` payload is machine surface, consumed by scripts and by the wire formats calendula sits over: the Calendar API answers `accessRole`, `timeZone` and `defaultReminders`, and JSCalendar and jCard spell their members the same way. Rendering the same concept `access-role` on the way out makes calendula the only party in the exchange spelling it that way.

A hyphenated key is also the one shape jq cannot address plainly: `.calendars[].accessRole` reads, `.calendars[]."access-role"` needs quoting, and every consumer pays that on every field.

## What changes

Every `--json` payload key becomes camelCase. Nine keys change spelling across the 23 registered commands: `displayName`, `calendarHomeSet`, `syncToken`, `timeZone`, `accessRole`, `defaultReminders`, `originalStart`, `calendarId` and `percentComplete`. Every other key is a single word and reads the same either way.

calendula is 0.1.0 and kebab-case never shipped, having landed in this same unreleased cycle, so there is no alias and no deprecation window.

## What does not change

The configuration vocabulary stays kebab-case: `home-dir`, `downloads-dir` and the table blocks are TOML a person writes, and they align with himalaya and cardamum. Nothing in `src/config.rs` is touched.

The plain-text rendering of every command is untouched, as are the top-level container keys (`calendars`, `items`, `todos`), which were already single words.

`event agenda` keeps its payload as it stands: its map is keyed by start instants rather than by field names, so there is no identifier in it to recase.
