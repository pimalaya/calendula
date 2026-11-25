---
cairn: change
id: events-alone
status: landed
created: 2026-11-25
---

# Calendula serves events alone

## Why

The unified API carries two families over the same resources: `event`, the projected VEVENT view, and `item`, the raw view of any iCalendar component. They overlap: `event read` already prints the iCalendar, `event add` and `event update` already take an iCalendar source, and `event delete` and `item delete` remove the same object. Two verbs with one meaning break the rule the family's unified APIs follow (one meaning per verb, himalaya cairn/spec/commands.md).

What `item` adds over `event` is reaching the VTODO and VJOURNAL of a CalDAV, vdir or pimdir calendar. Those belong to unfog and journal (spec: "One family per stored component kind"), the `todo` and `journal` commands already left for them in this release, and alarms fire in carillon. On Google Calendar and Microsoft Graph, which store events alone, `item` is `event` without the projection.

Cardamum has one family, `card`, giving both the typed row and the raw vCard, and Himalaya pairs `envelope` and `message` over one object. Calendula's counterpart of `card` is `event`. Removing `item` after 1.0 is a major release, so it goes now, before 1.0.0 is tagged.

## What

The unified API is `calendar` and `event`. Raw access to any component stays where it is protocol-specific: `caldav get`, `caldav put` and `caldav report query --component`, and `vdir item`.

### Decisions

- **The `item` family is removed**: `item list|read|add|update|delete|build`, its hidden aliases (`items`, `item create`), its REPL commands, its MCP tools (`item_list`, `item_read`, `item_add`, `item_update`, `item_delete`) and its schemas (`calendula-item-*`). A command line naming it is refused by clap as any unknown command, no hidden alias kept, as `todo` and `journal` were.
- **The capabilities are named after `event`**: `item.list`, `item.read`, `item.add`, `item.update`, `item.delete`, `item.update.if-match` and `item.delete.if-match` become `event.list`, `event.read`, `event.add`, `event.update`, `event.delete`, `event.update.if-match` and `event.delete.if-match`, as cardamum names its `card.*`; `account capability` and the `capability-missing` messages print the new names.
- **The configuration follows**: `item.composer` becomes `event.composer` (top level and per account), and `item.list.page-size` goes. Both former keys are refused by serde as unknown keys, never by name (config-unknown-keys-serde-only).
- **The error codes name events**: `item-not-found`, `item-unreadable` and `item-invalid` become `event-not-found`, `event-unreadable` and `event-invalid`, as cardamum's `card-*`. `item-duplicate` stays: it is io-pimdir's code, which every pimdir app relays under that name.
- **A non-event resource is never touched through `event`**: an id naming a resource that holds no VEVENT is `event-not-found` for `event read`, `event update` and `event delete`, so a VTODO sharing a calendar is left intact. A listing skips it, as it does today. Today `event delete` removes such a to-do and `event update <id> <ICAL>` overwrites it.
- **An event write takes an event**: an iCalendar source holding no VEVENT is refused `event-invalid` by `event add` and `event update`, before anything is reached, so `event` never writes a to-do or a journal entry. Bytes that do not parse still go to the backend as written.
- **Every write is the user's**: with `item` gone, nothing writes a body the user did not write, so the pimdir backend queues every body whole (`enqueue_user_body`) and the `ItemOrigin` plumbing goes, with its `Foreign` stripping.
- **The internals keep their names**: `shared::item` (`CalendarItem`, the calendar object resource the backends read and write) stays, the event family being a projection over it. Only the command surface, the capability and code names and the configuration move.

### Out of scope

- unfog and journal themselves: they ship on their own schedule; until then a CalDAV to-do is reachable through `caldav get` and `caldav put`.
- `vdir item`, which stays the raw, byte-for-byte view of a vdir collection.
