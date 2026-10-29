---
cairn: delta
change: events-only
---

## ADDED Requirements

### Requirement: A leftover todo or journal table is ignored with a warning

A `todo` or `journal` table left in a configuration, at the top level or under an account, SHALL be ignored, and calendula SHALL warn once per table, naming the CLI that took the domain over (unfog for `todo`, the journal CLI for `journal`). It SHALL NOT be refused: the top-level and account blocks do not deny unknown keys, and a configuration that worked SHALL keep loading after an upgrade.

#### Scenario: An upgraded configuration
- GIVEN a configuration written for 0.2 with a `[todo.list]` table and an `[accounts.work.journal.list]` table
- WHEN any calendula command loads it
- THEN the command runs, and two warnings name unfog and the journal CLI

## MODIFIED Requirements

### Requirement: The component families against item
The shared API SHALL offer two kinds of view over the same resources. `item` is the raw, unfiltered one: it lists, reads, writes and deletes any iCalendar object by id, leaving the bytes untouched, a to-do or a journal entry included.

`event` is the projected one: it keeps VEVENT, renders the columns an event is read by, and draws a cal(1)-style agenda.

Both SHALL share the `-k/--calendar` selector and the same item API; only the rendering and the filter differ.

### Requirement: One family per stored component kind
calendula SHALL be the events CLI: the shared API SHALL offer one projected family, `event`, for VEVENT. VTODO belongs to unfog and VJOURNAL to the journal CLI, one CLI per iCalendar domain, since CalDAV holds one component type per calendar object resource (RFC 4791 4.1) and the providers run events, tasks and notes as separate services.

`event` SHALL cover `list`, `read`, `create`, `update` and `delete` over the item API. An item whose bytes do not parse SHALL project nothing rather than fail the listing.

#### Scenario: A to-do in an event calendar
- GIVEN a CalDAV calendar holding a VEVENT and a VTODO
- WHEN `calendula event list` and `calendula item list` run on it
- THEN the event listing shows the VEVENT alone, and the item listing shows both

### Requirement: Columns follow the kind
An event listing SHALL render its summary and its start and end.

The event table SHALL show the start and end for a person to read, `startsAt` and `endsAt` in the local zone (`TZ`, then the system's), the one a floating time is read in: `2026-10-30 09:00` for a timed event, the days for a whole-day one with the end as its last day (`2026-05-19`), never before its first, so a zero-length one (`DTEND` equal to `DTSTART`) ends on its start. A time that did not resolve SHALL show as iCalendar spells it. Squeezed into the terminal or `--max-width`, the table SHALL truncate the id and the summary but keep the start and end whole. The `--json` payload SHALL stay as it is, `start` and `end` iCalendar-spelled and a whole day's `endsAt` exclusive.

A property the event omits SHALL render empty rather than absent, so the columns line up down the table.

### Requirement: Only events draw an agenda
`agenda` SHALL stay a VEVENT command. The grid marks the days that carry an event because an event occupies time.

### Requirement: A component window is applied locally
`event list` pushes its window down where the backend can narrow server-side. `item list` takes no window.

### Requirement: A delete can be gated on a version
`event delete` and `item delete` SHALL take `--if-match <ETAG>`. CalDAV SHALL send it as `If-Match`, msgraph SHALL compare it with the event's `changeKey`, and pimdir SHALL check it as an update does. A backend that cannot gate a delete (vdir, gcal) SHALL refuse the flag rather than drop it.

`item read` SHALL add the `etag` of the version read to its JSON, `null` when the backend has none.

### Requirement: A minted item carries an identity, and its family names its kind
An item minted from nothing SHALL be a VCALENDAR carrying a PRODID and one VEVENT, carrying a UID minted as a fresh UUID, a DTSTAMP of now and a DTSTART of now, so the seed passes the check it will be held to the moment the composer hands it back. The values are placeholders, and an event with no time is not an event. An editor handed an empty file mints no identity, and the CalDAV resource name and the pimdir link id both derive from the UID.

`item` names no component kind, so it SHALL mint nothing: `item build` and `item create` given no source SHALL bail, naming `event` as the family that can, and unfog and the journal CLI for a to-do and a journal entry.

### Requirement: Backend blocks
An account SHALL carry an optional block per compiled backend: `vdir` with a `home-dir`, `pimdir` with a `root` and an optional `account`, `caldav` with its endpoint, TLS and authentication, `gcal` with TLS and authentication.

An account MAY carry several; which one a shared command uses is the backend capability's business.

Every listing family SHALL additionally carry a rendering block naming its default page size and its column colours: `calendar.list`, `event.list` and `item.list`.

## REMOVED Requirements

- The `todo` and `journal` families: their `list`, `read`, `create`, `update`, `delete` and `build` commands, their projections and tables, from "One family per stored component kind" and "Columns follow the kind" (the todo columns and the reading of `STATUS:COMPLETED` as fully done move to unfog's spec).
- The local window of `todo list` and `journal list`, from "A component window is applied locally": unfog and the journal CLI own their listings (the journal CLI pushes a range down against `DTSTART`, RFC 4791 9.9).
- The `todo.list` and `journal.list` rendering blocks, from "Backend blocks", and the `todo` and `journal` entries of the wizard's render order.
- The minted VTODO and VJOURNAL, from "A minted item carries an identity, and its family names its kind".
