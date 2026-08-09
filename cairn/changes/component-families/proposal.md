---
cairn: change
id: component-families
status: landed
created: 2026-08-09
---

# One shared family per component kind

## Why

A calendar collection mixes component kinds, and calendula offers two views over them: `item`, the raw one that lists and edits any iCalendar object by id, and `event`, the VEVENT one that projects summary and time columns and draws an agenda. RFC 5545 defines two further kinds a calendar actually stores, and neither has a view: a VTODO is a task with a due date, a completion percentage and a priority, and a VJOURNAL is a dated note.

Today they are reachable only through `item`, which renders an id, an ETag and a byte count. A user with a task list gets no due date, no status, no way to tell a finished task from a pending one without reading the raw bytes of every resource. The projection that makes `event list` useful is exactly what is missing, and the gap is a rendering gap, not a protocol one: every backend already stores these components, and CalDAV servers that accept them advertise them in their `supported-calendar-component-set`.

## What

Two families, `todo` and `journal`, joining `event` as component views over the same item API, with the same five verbs (`list`, `read`, `create`, `update`, `delete`) and the same `-k/--calendar` selector.

They are views, not new backend operations. Nothing reaches [`CalendarClient`](../../src/shared/client.rs): each family lists items, projects the component kind it owns, drops the rest, and renders the columns that kind carries. That is what `event` already does, so adding a kind costs a projection and a table and touches no backend.

The columns follow what each kind is for:

- **todo**: SUMMARY, DUE, STATUS, PRIORITY and the completion percentage, the five properties a task list is read by.
- **journal**: SUMMARY and DTSTART, since a journal entry is a dated note and carries no end.

No agenda for either. The grid marks the days that carry an event because an event occupies time; a task's due date and a note's date do not fill a day, and a cal(1) grid of them would suggest they do.

## Scope

In: the two projections, the two command families, their configuration blocks (page size and column colours, as `event` and `item` already have), the CLI wiring, the spec and the tests.

Out: VFREEBUSY and VTIMEZONE. The first is the answer to a query, not a resource a calendar stores, and where a provider offers it (Google does) it belongs to that backend's own family. The second is a supporting component that defines the zones the others reference; it is not an item a user lists.

Also out: a `done` verb marking a task complete. It reads as a natural next step, but completing a task is a read-modify-write of STATUS and PERCENT-COMPLETE, which is a template concern; calendula edits iCalendar by supplying iCalendar, and a partial-update verb would be the first exception to that.
