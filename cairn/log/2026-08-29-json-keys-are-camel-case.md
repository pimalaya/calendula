---
cairn: log
date: 2026-08-29
change: json-keys-are-camel-case
---

# The `--json` keys read as the wire formats they wrap

Landed the same day as [config-and-output-alignment](2026-08-29-config-and-output-alignment.md), which had settled the payload keys on kebab-case by reasoning from the configuration schema. The family moved the other way: a `--json` payload is machine surface and belongs with the wire formats, not with the TOML a person writes.

## What landed

**Forty-one `#[serde(rename_all)]` attributes moved from kebab-case to camelCase.** The 22 `*Output` types carrying one, the twelve nested row types (`CollectionRow`, `BackendCheck`, `AccountRow`, the two `CalendarRow`s, `ColorRow`, `RuleRow`, `CalendarBusy`, `BusyPeriod`, `SettingRow`, `InstanceRow`, `PimdirCalendarStatus`), and the seven shared model types the listings serialize ([src/shared/calendar.rs](../../src/shared/calendar.rs), [src/shared/item.rs](../../src/shared/item.rs), [src/shared/event.rs](../../src/shared/event.rs), [src/shared/todo.rs](../../src/shared/todo.rs), [src/shared/journal.rs](../../src/shared/journal.rs)).

Nine keys actually changed spelling: `displayName` (`vdir list`, `caldav list`), `calendarHomeSet` (`caldav discover`), `syncToken` (`caldav list`), `timeZone`, `accessRole` and `defaultReminders` (`gcal calendars`), `originalStart` (`gcal instances`), `calendarId` (`item list`) and `percentComplete` (`todo list`). Every other key is one word.

**[src/config.rs](../../src/config.rs) is untouched.** Its 32 attributes and the four clap `#[command(rename_all)]` in the backend `cli.rs` files stay kebab-case: the first is the TOML vocabulary himalaya and cardamum share, the second is how a subcommand is spelled on the command line.

**`EventAgendaOutput` needed no recasing.** Its hand-written `Serialize` emits a map keyed by start instants and its hand-written `JsonSchema` describes a `BTreeMap<String, Vec<String>>`, so neither names a field. The two still agree, and the published schema is unchanged.

**The `JSON output:` help lines were rewritten** in the eight commands whose doc comment spells a changed key, keeping the help the usage reference it is meant to be.

## Verification

`cargo fmt`, `cargo check`, `cargo test` (93 tests) and `cargo clippy` clean under `--all-features` and under each backend feature alone. `json-schema --dir` regenerated all 23 files and every property name in them is camelCase.

## Capabilities moved

- commands: the casing of a `--json` payload key
