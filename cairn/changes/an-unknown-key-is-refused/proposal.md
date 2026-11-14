---
cairn: change
id: an-unknown-key-is-refused
status: landed
created: 2026-11-14
---

# An unknown key is refused

> Cross-repo change, same id in Cardamum (8f25abf). Calendula follows it.

## Why

The owner decided on 2026-11-14 that a configuration key a binary does not know is refused at every level, as Himalaya does. Calendula refused one in its option and backend tables, but left its root and its accounts lenient on purpose, so a reduced build would load a file naming a backend it leaves out: a typo there (`defualt = true`, `calender.default`) loaded silently. The tables 0.2 read for colours (`calendar.list`, `event.list.table`, `item.list.table`) were refused generically, as unknown fields of their block, where `todo`, `journal`, `table` and the top-level `account` are refused by name.

## What

- `Config` and `AccountConfig` refuse an unknown key (`deny_unknown_fields`), as every other table already did. No table flattens another, so none needs restructuring.
- The block of a backend the build leaves out (`vdir`, `pimdir`, `caldav`, `gcal`, `msgraph`) is an `IgnoredAny` field of `AccountConfig` under `cfg(not(feature))`, so it loads unread and one file still serves every build. Calendula has no `proxy` key, so nothing else needs it.
- `calendar.list`, `event.list.table` and `item.list.table` are refused by name, at the root and in an account, as the other removed tables are, saying the output is plain text and to remove them.
- Spec: config ("Unknown fields", "A leftover todo or journal table is refused by name", "No colour or table is configured", "One sample configuration, with a section per backend").
