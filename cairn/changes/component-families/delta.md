---
cairn: change
change: component-families
---

# Delta

## ADDED Requirements

### Requirement: One family per stored component kind
The shared API SHALL offer one command family per iCalendar component kind a calendar stores: `event` for VEVENT, `todo` for VTODO and `journal` for VJOURNAL. Each SHALL cover `list`, `read`, `create`, `update` and `delete`, share the `-k/--calendar` selector, and read the same item API, so a component family costs a projection and a table and adds no backend operation.

A component family SHALL keep only its own kind and drop the others, `item` remaining the unfiltered view. An item whose bytes do not parse SHALL project nothing rather than fail the listing.

### Requirement: Columns follow the kind
A component listing SHALL render the properties its kind is read by: an event its summary and its start and end, a todo its summary, due date, status, priority and completion percentage, a journal its summary and its date. A property the component omits SHALL render empty rather than absent, so the columns line up down the table.

### Requirement: Only events draw an agenda
`agenda` SHALL stay a VEVENT command. The grid marks the days that carry an event because an event occupies time; a todo's due date and a journal's date do not fill a day, and drawing them the same way would say they do.

### Requirement: Kinds with no family
VFREEBUSY and VTIMEZONE SHALL NOT get a family. A VFREEBUSY is the answer to a query rather than a resource a calendar stores, and where a backend offers such a query it belongs to that backend's own protocol-specific family. A VTIMEZONE defines the zones the other components reference and is not an item a user lists.

## MODIFIED Requirements

### Requirement: event against item
The shared API SHALL offer two kinds of view over the same resources. `item` is the raw, unfiltered one: it lists, reads, writes and deletes any iCalendar object by id, leaving the bytes untouched. The component families (`event`, `todo`, `journal`) are the projected ones: each keeps its own component kind, renders the columns that kind is read by, and `event` additionally draws a cal(1)-style agenda. All SHALL share the `-k/--calendar` selector and the same item API; only the rendering and the filter differ.

### Requirement: Backend blocks
An account SHALL carry an optional block per compiled backend: `vdir` with a `home-dir`, `pimdir` with a `root` and an optional `source`, `caldav` with its endpoint, TLS and authentication, `gcal` with TLS and authentication. An account MAY carry several; which one a shared command uses is the backend capability's business.

Every listing family SHALL additionally carry a rendering block naming its default page size and its column colours: `event.list`, `todo.list`, `journal.list`, `item.list` and `calendar.list`.
