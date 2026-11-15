---
cairn: change
id: events-only
status: landed
created: 2026-10-27
---

# calendula is the events CLI

## Why

calendula has one shared family per stored component kind (component-families, 2026-08-09): `event`, `todo` and `journal`. Pimalaya now splits the iCalendar components by domain, one CLI owning each (decided 2026-10-27):

- **calendula**: VEVENT, with freebusy and VAVAILABILITY as backend-specific commands.
- **unfog**: VTODO.
- **journal** (crate `journal`, "CLI to manage journal entries"): VJOURNAL.
- an **alarm daemon** firing the VALARMs of events and tasks.

The split follows the standards and the providers. CalDAV allows one component type per calendar object resource (RFC 4791 4.1), so every stored object belongs to exactly one domain. Google, Microsoft and Apple run events, tasks and notes as separate services, and the `todo` and `journal` families already refuse the gcal and msgraph backends for that reason. Two CLIs editing VTODO would mean two shared surfaces over the same object, each bounded by different backends (unfog's To Do and Google Tasks backends, which calendula will never have).

The draft waited on unfog and journal releasing their families. The owner unblocked it on 2026-11-12, as change 2 of the-family-standards: calendula becomes the events CLI now, and neither unfog nor journal being released yet is accepted.

## What

- `todo` and `journal` leave calendula: their command families, projections, configuration blocks (`todo`, `journal` at the top level and per account), their `json-schema` entries and the wizard's render order entries. Their domains go to unfog and journal.
- `event`, `agenda`, `calendar` and the backend families stay as they are.
- `item` stays the raw view over any calendar object by id, bytes untouched, a VTODO or a VJOURNAL included: it is the escape hatch for a mixed collection, and removing it buys nothing. The pimdir refusal of a to-do written into a calendar taking none stays, reached through `item`. `item build` and `item create` given no source name `calendula event`, `unfog` and `journal` as what composes an event, a to-do and a journal entry.
- `pimdir search` and `pimdir count` lose `--kind`, which could only name `event` once the families left: they search the account's events. Unreleased (added after 0.2.0), so nothing to migrate.
- A leftover `todo` or `journal` table in a configuration is refused by name, naming the CLI that took it over (owner, 2026-11-11: the family refuses removed configuration keys). The top-level and account blocks keep denying no unknown key, so the refusal is two explicit fields whose deserialization always fails. The 2026-10-27 draft ignored such a table with a warning; that was dropped for the family rule.
- `caldav create` keeps its `--component` flag. The shared `calendar create` stays as it is, checked on each backend on 2026-11-12: CalDAV sends no `supported-calendar-component-set`, so the server's default applies; vdir makes a directory taking any kind; pimdir refuses to create a calendar (its sync engine does); gcal and msgraph calendars hold events alone. Restricting the CalDAV set to VEVENT would leave `item` unable to store a to-do in a calendar calendula created, and is left to the owner.

This is breaking: the next release is 0.3.0 (the version stays the owner's call), with MIGRATION.md pointing `calendula todo` to unfog and `calendula journal` to journal.

## Out

- Alarm flags: change event-alarm-flag.
- Freebusy and VAVAILABILITY in the shared API: freebusy is a different call per provider (CalDAV scheduling outbox, `gcal free-busy`, Graph `getSchedule`), and VAVAILABILITY (RFC 7953) is rare on servers; both stay backend-specific.
- Narrowing the listing query's `kind` (any `IcalComponentKind`) to events: the backends keep the generic kind filter, `item` asking for none and `event` for VEVENT.
