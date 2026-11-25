---
cairn: delta
change: events-alone
---

# Delta

## MODIFIED Requirements

### commands.md: One family per stored component kind
calendula SHALL be the events CLI: the unified API SHALL be `calendar` and `event`, `event` the one family over the calendar object resources, VEVENT alone. VTODO belongs to unfog and VJOURNAL to journal, one CLI per iCalendar domain. `event` SHALL cover `list`, `read`, `search`, `add`, `update`, `delete`, `build` and `agenda`, `event read` printing the projected row and the resource's iCalendar as cardamum's `card read` prints the vCard. Raw access to any other component SHALL stay protocol-specific: `caldav get`, `caldav put`, `caldav report query --component` and `vdir item`.

#### Scenario: A to-do in an event calendar
- GIVEN a CalDAV calendar holding a VEVENT and a VTODO
- WHEN `calendula event list` runs on it
- THEN the listing shows the VEVENT alone
- AND `calendula event read`, `event update` and `event delete` naming the VTODO's id fail `event-not-found`, the VTODO untouched

### commands.md: An event is named by its id alone
`event update` given a source SHALL read the id first, as `event delete` does. An id naming a resource that holds no VEVENT SHALL fail `event read`, `event update` and `event delete` coded `event-not-found`, the resource untouched; bytes that do not parse pass.

### commands.md: An event write takes an event
An iCalendar source holding no VEVENT SHALL be refused `event-invalid` by `event add` and `event update` before anything is reached; bytes that do not parse go to the backend as written.

### backends.md: pimdir refuses an online meeting no source can create
Every body SHALL be queued whole (`enqueue_user_body`), its `X-PIMDIR-ONLINE-MEETING` the user's ask; the foreign-body stripping goes with `item`.

### commands.md: A page of a family holds that family
The sentence on the raw `item` family is removed: every unified listing narrows to VEVENT before it paginates.

### commands.md: A component window is applied locally
`event list` pushes its window down where the backend can narrow server-side, and applies it after parsing elsewhere. A component carrying no date at all SHALL never show in a windowed listing.

### commands.md: Capability registry
The capabilities SHALL be `calendar.*`, `event.*` (`event.list`, `event.read`, `event.search`, `event.add`, `event.update`, `event.delete`, `event.update.if-match`, `event.delete.if-match`) and the scheduling ones; no `item.*` capability exists.

### commands.md: A JSON error carries a stable code
The resource codes SHALL be `event-not-found`, `event-unreadable` and `event-invalid`, as cardamum names its `card-*`; `item-duplicate` SHALL stay io-pimdir's code, relayed under that name.

### config.md: The composer
`event.composer` SHALL hold the command an event is edited through, at the top level and per account, as a shell line or an argv list, the path appended. `item.composer` and `item.list.page-size` SHALL be refused by serde as unknown keys.

### mcp.md: One tool per leaf
The tools SHALL be those of `account`, `calendar` and `event`; `--allow-write` SHALL add the calendar and event writes.

## REMOVED Requirements

### commands.md: The component families against item
The `item` family and its raw view over any component are removed from the unified API.
