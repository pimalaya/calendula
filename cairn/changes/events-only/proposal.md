---
cairn: change
id: events-only
status: draft
created: 2026-10-27
---

# calendula is the events CLI

## Why

calendula has one shared family per stored component kind (component-families, 2026-08-09): `event`, `todo` and `journal`. Pimalaya now splits the iCalendar components by domain, one CLI owning each (decided 2026-10-27):

- **calendula**: VEVENT, with freebusy and VAVAILABILITY as backend-specific commands.
- **unfog**: VTODO.
- a **journal CLI** (working name): VJOURNAL.
- an **alarm daemon** firing the VALARMs of events and tasks.

The split follows the standards and the providers. CalDAV allows one component type per calendar object resource (RFC 4791 4.1), so every stored object belongs to exactly one domain. Google, Microsoft and Apple run events, tasks and notes as separate services, and the `todo` and `journal` families already refuse the gcal and msgraph backends for that reason. Two CLIs editing VTODO would mean two shared surfaces over the same object, each bounded by different backends (unfog's To Do and Google Tasks backends, which calendula will never have).

## What

- `todo` and `journal` leave calendula: their command families, projections, configuration blocks (`todo`, `journal` at the top level and per account) and wizard render order. Their code moves to unfog and to the journal CLI, which land first so nobody is left without a command.
- `event`, `agenda`, `calendar` and the backend families stay as they are.
- `item` stays the raw view over any calendar object by id, bytes untouched: it is the escape hatch for a mixed collection, and removing it buys nothing.
- A leftover `[todo]` or `[journal]` table in a configuration is ignored with a warning naming the CLI that took it over, rather than refused: the top-level and account blocks do not deny unknown keys, and a configuration must not break on upgrade.
- `caldav create` keeps its `--component` flag; the shared `calendar create` makes event calendars.

This is breaking: the next release is 0.3.0 (the version stays the user's call), with MIGRATION.md pointing `calendula todo` to unfog and `calendula journal` to the journal CLI.

## Out

- Alarm flags: change event-alarm-flag.
- Freebusy and VAVAILABILITY in the shared API: freebusy is a different call per provider (CalDAV scheduling outbox, `gcal free-busy`, Graph `getSchedule`), and VAVAILABILITY (RFC 7953) is rare on servers; both stay backend-specific.
