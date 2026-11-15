---
cairn: delta
change: events-only
---

## ADDED Requirements

### Requirement: A leftover todo or journal table is refused by name
A `todo` or `journal` table left in a configuration, at the top level or under an account, SHALL fail the load, naming the CLI that took the domain over (unfog, the Pimalaya CLI for tasks, for `todo`; journal, the Pimalaya CLI for journal entries, for `journal`) and saying to remove the table. The top-level and account blocks still deny no unknown key: these two tables are refused by name, as the family refuses its removed keys, rather than read or ignored.

#### Scenario: An upgraded configuration
- GIVEN a configuration written for 0.2 with a `[todo.list]` table
- WHEN any calendula command loads it
- THEN the command fails before reaching a backend, the error locating the table (its line, or its key once several files are merged) and naming unfog

#### Scenario: A table under an account
- GIVEN a configuration with an `[accounts.work.journal]` table
- WHEN any calendula command loads it
- THEN the command fails, the error naming journal

## MODIFIED Requirements

### Requirement: The component families against item
The shared API SHALL offer two kinds of view over the same resources. `item` is the raw, unfiltered one: it lists, reads, writes and deletes any iCalendar object by id, leaving the bytes untouched, a to-do or a journal entry included.

`event` is the projected one: it keeps VEVENT, renders the columns an event is read by, and draws a cal(1)-style agenda.

Both SHALL share the `-k/--calendar` selector and the same item API; only the rendering and the filter differ.

### Requirement: One family per stored component kind
calendula SHALL be the events CLI: the shared API SHALL offer one projected family, `event`, for VEVENT. VTODO belongs to unfog and VJOURNAL to journal, one CLI per iCalendar domain, since CalDAV holds one component type per calendar object resource (RFC 4791 4.1) and the providers run events, tasks and notes as separate services.

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
`event list` pushes its window down where the backend can narrow server-side, and applies it after parsing elsewhere. `item list` takes no window.

A component carrying no date at all SHALL show only in an unfiltered listing.

### Requirement: A delete can be gated on a version
`event delete` and `item delete` SHALL take `--if-match <ETAG>`. CalDAV and gcal SHALL send it as `If-Match`, msgraph SHALL compare it with the event's `changeKey`, and pimdir SHALL check it as an update does. A backend that cannot gate a delete (vdir) SHALL refuse the flag rather than drop it.

`item read` SHALL add the `etag` of the version read to its JSON, `null` when the backend has none.

### Requirement: A minted item carries an identity, and its family names its kind
An item minted from nothing SHALL be a VCALENDAR carrying a PRODID and one VEVENT, carrying a UID minted as a fresh UUID, a DTSTAMP of now and a DTSTART of now, so the seed passes the check it will be held to the moment the composer hands it back. The values are placeholders, and an event with no time is not an event. An editor handed an empty file mints no identity, and the CalDAV resource name and the pimdir link id both derive from the UID.

`item` names no component kind, so it SHALL mint nothing: `item build` and `item create` given no source SHALL bail, naming `calendula event` for an event, `unfog` for a to-do and `journal` for a journal entry.

### Requirement: An event build answers its projection
`event build` under `--json` SHALL answer `{contents, method, events}`: the iCalendar it built, the calendar's `METHOD` uppercased (`null` when it carries none), and every VEVENT projected as `event read` projects them, the item id empty and the etag `null`. Its text output SHALL stay the iCalendar alone. `item build` SHALL keep answering `{contents}`.

### Requirement: pimdir searches calendar items through the store's index
`calendula pimdir search <query>...`, gated on the `pimdir` feature, SHALL answer the pimdir SEARCH query language (§8) from the store's search index, scoped to the account's events: the query is read as its conjuncts plus `kind:event`, `account:<account>` when the store groups the account under a name, and `collection:<id>` when `-k/--calendar` names a calendar, each a top-level conjunct. Without `-k` it SHALL search every calendar of the account, `calendar.default` not applied; a `-k` naming no calendar of the account SHALL be refused, naming those it holds. The words of the query are joined by spaces into one query. Calendula SHALL open the index read-only (io-pimdir `PimdirIndex::open_query`) and SHALL NOT create, rebuild nor refresh it: the store's owner, its sync engine, does. The answer SHALL be the store as of the owner's last refresh: a queued write is not listed nor folded in.

A `date:` term SHALL compare the occurrences the index expanded (SEARCH §7), its days, `today` and `yesterday` read in the zone the tables read times in (`TZ`, then the system's), stated to the index as an IANA name (SEARCH §8.1); a zone with no IANA name SHALL read them in UTC, with a warning.

Each hit SHALL render with the `event list` columns (Columns follow the kind), from its summary when its body is not local, then its calendar, what a text term matched (`body`, `summary`, or `null` when only field terms did) and a `snippet`; its `id` is the public `seq`, which `event read` takes. `--sort date|name|relevance` orders the hits, `date` (latest first) by default, a series by its first occurrence, and `--page` and `--page-size` page them, the page size defaulting to `event list`'s. Under `--json` the page carries `hits`, `more` (whether hits follow it) and `coverage`, each hit the event projection plus `calendarId`, `matched` and `snippet`.

#### Scenario: A series found by a later occurrence
- GIVEN an indexed pimdir calendar holding a weekly event begun on 2019-01-07, a Monday
- WHEN `calendula pimdir search date:2026-11-09` runs
- THEN the event is listed

#### Scenario: A day is the local day
- GIVEN `TZ=Europe/Paris` and an event at 00:30 on 2026-11-07, its `DTSTART` in `Europe/Paris`
- WHEN `calendula pimdir search date:2026-11-07` runs
- THEN the event is listed, and `date:2026-11-06` does not list it

#### Scenario: An all-day event stays on its day
- GIVEN `TZ=Europe/Paris`, an index reading floating times in `Europe/Paris`, and an all-day event on 2026-11-07
- WHEN `calendula pimdir search date:2026-11-08` runs
- THEN the event is not listed

#### Scenario: A queued event is not listed
- GIVEN an event created in a pimdir calendar and still in the queue
- WHEN `calendula pimdir search` runs a query its body matches
- THEN it is not listed until the owner applies the create and refreshes the index

#### Scenario: A to-do is not an event
- GIVEN an indexed pimdir calendar holding an event and a to-do both naming the budget
- WHEN `calendula pimdir search budget` runs
- THEN the event alone is listed

### Requirement: Backend blocks
An account SHALL carry an optional block per compiled backend: `vdir` with a `home-dir`, `pimdir` with a `root` and an optional `account`, `caldav` with its endpoint, TLS and authentication, `gcal` with TLS and authentication.

An account MAY carry several; which one a shared command uses is the backend capability's business.

Every listing family SHALL additionally carry a rendering block naming its default page size and its column colours: `calendar.list`, `event.list` and `item.list`.

### Requirement: The composer is a command, not a library
`item.composer` SHALL hold the command an item is edited through, at the top level and per account, as a shell line or an argv list. calendula SHALL spawn it rather than embed an editor: what edits an iCalendar is the user's choice, and a terminal editor, a graphical one and a purpose-built one are all one configuration line apart.

The key SHALL sit under `item` rather than under `event`: `event` and `item` are views over the same resources, and what is edited is an iCalendar object, which is what `item` names.

### Requirement: A queued write is reported as queued
A backend whose writes are queued for a sync to push, rather than made on the server, SHALL have its writes confirmed as queued: `create` as "`<Kind>` `<id>` queued for the next sync", `update` as "`<Kind>` `<id>` update queued for the next sync", `delete` as "`<Kind>` deletion queued for the next sync", for the `event` and `item` families. Every other backend SHALL keep "successfully created", "updated", "deleted". The JSON payloads SHALL be the same for every backend.

pimdir is the only backend that queues its writes.

## REMOVED Requirements

(none whole: the todo and journal parts leave the requirements modified above)

- The `todo` and `journal` families: their `list`, `read`, `create`, `update`, `delete` and `build` commands, their projections and tables, from "One family per stored component kind" and "Columns follow the kind" (the todo columns and the reading of `STATUS:COMPLETED` as fully done move to unfog's spec).
- The local window of `todo list` and `journal list`, from "A component window is applied locally": unfog and journal own their listings.
- `--kind event|todo|journal` of `pimdir search` and `pimdir count`, and the to-do and journal hit columns, from "pimdir searches calendar items through the store's index".
- The `todo.list` and `journal.list` rendering blocks, from "Backend blocks", and the `todo` and `journal` entries of the wizard's render order.
- The minted VTODO and VJOURNAL, from "A minted item carries an identity, and its family names its kind".
- The `todo` and `journal` families from "A delete can be gated on a version", "An event build answers its projection" and "A queued write is reported as queued".
