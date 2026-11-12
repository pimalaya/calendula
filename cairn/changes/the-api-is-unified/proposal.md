---
cairn: change
id: the-api-is-unified
status: landed
created: 2026-11-12
---

# The API is unified

## Why

The owner renamed the cross-protocol API the "unified API" on 2026-11-12 (himalaya the-api-is-unified, b21c2d7f), "shared API" suggesting a least common denominator, and asked calendula to follow the same day. calendula adopts the capability union in capability-shared-api (the-family-standards, change 9); the name comes first so every change in between already reads the new word.

## What

- "shared API" reads "unified API" across the code, its `--help` text (`Manage calendars using the unified API.`, `Manage events (VEVENT) using the unified API.`, `Manage raw iCalendar items (any component) using the unified API.`), README.md, config.sample.toml, the spec and the active jmap-backend change; so do the phrases naming the API's parts: its commands, writes, operations, outputs, surface and command families.
- Renamed requirements, their text otherwise unchanged: "The unified API is a strict least common denominator" (commands); "Unified operation set" and "Unified outputs carry no backend details" (backends).

## Scope / non-goals

No behaviour changes. The least-common-denominator rule stands, under its new name, until capability-shared-api replaces it. "Shared" stays where it means shared by the code: the src/shared module path, the shared types (`Calendar`, `CalendarItem.contents`), the shared client, the shared `paginate`, pimalaya-cli's shared REPL. The log, landed changes, released CHANGELOG sections and the 0.2 part of MIGRATION.md keep the words of their time; the Unreleased CHANGELOG section names no shared API.
