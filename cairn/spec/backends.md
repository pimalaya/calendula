---
cairn: spec
capability: backends
status: current
---

# Backends

Each backend is a `<Proto>Backend` adapter in `src/<proto>/backend.rs`, implementing the shared operations over one io-* crate's high-level client and converting its results into calendula's own shared types (`Calendar`, `CalendarDiff`, `CalendarItem`, `CalendarTimeRange`). calendula owns those types; no aggregator library sits between it and the io-* crates.

### Requirement: Shared operation set
The shared adapters SHALL cover, per backend: `list_calendars`, `create_calendar`, `update_calendar`, `delete_calendar`, `list_items`, `get_item`, `create_item`, `update_item` and `delete_item`. A backend that cannot model an operation SHALL refuse it with a message naming what to do instead, rather than emulating it.

### Requirement: A create reports the identifier it was given
`create_calendar` and `create_item` SHALL return the identifier the backend actually assigned, and the command SHALL report that one. It is the requested id on every backend that lets a client name a collection, and a server-minted one where it does not, so a create never names a resource that does not exist.

### Requirement: Backend selection order
The shared commands SHALL target the backend the global `--backend` flag selects. Its default, `auto`, SHALL take the first configured-and-compiled backend in the order vdir, pimdir, CalDAV, gcal, preferring a local read to a network round-trip and a protocol-standard server to a vendor API. A named value SHALL pin the command to that backend and bail when the account carries no matching configuration block. The protocol-specific commands SHALL ignore the flag.

### Requirement: CalDAV backend
CalDAV SHALL adapt io-webdav's RFC 4791 surface over a connected client whose calendar home-set is already resolved. Item ids SHALL be the resource names the server returned, verbatim: io-webdav neither appends nor strips a file extension, so an id a listing showed addresses the same resource on every verb. A create SHALL propose a resource name derived from the item's own UID when that UID is URL-safe, falling back to a content digest, and SHALL keep the id the server reports in its `Location` header when it names the resource itself.

### Requirement: CalDAV pushes the time range down
A [`CalendarTimeRange`](#requirement-time-range-filtering) SHALL reach CalDAV as an RFC 4791 `time-range` filter nested in a VEVENT `comp-filter`, so the server does the narrowing. The filter SHALL scope to VEVENT only, since RFC 4791 9.9 defines the overlap test against a component's own start and end, and a VTODO or VJOURNAL carrying neither would be dropped for the wrong reason.

### Requirement: Google Calendar backend
gcal SHALL adapt io-gcal's Calendar API v3 client. A calendar is a calendar list entry and an item is one calendar object resource of it: a single event, or a recurring event together with the exceptions that modify its instances (see [A recurrence set is one resource](./projection.md#requirement-a-recurrence-set-is-one-resource)). Item ids SHALL be the event ids the API returned, verbatim, and the id of a set SHALL be the master's. `create_item` SHALL use `events.import` when the projected event carries an iCalendar UID, so the UID survives, and `events.insert` otherwise. `update_item` SHALL honour `if_match` as the API's `If-Match` header, so the optimistic concurrency Google offers is not silently dropped.

`create_calendar` SHALL insert a secondary calendar. Google mints the id of a calendar it creates, so the requested id SHALL NOT be honoured and the [assigned one](#requirement-a-create-reports-the-identifier-it-was-given) SHALL be reported instead. `update_calendar` SHALL patch the calendar for the name and the description and the calendar list entry for the colour, since Google splits those across the two resources; a Google calendar always carries a colour, so clearing one SHALL be refused by name.

### Requirement: gcal pushes the time range down
A [`CalendarTimeRange`](#requirement-time-range-filtering) SHALL reach gcal as the `timeMin` and `timeMax` parameters of `events.list`, so the server does the narrowing. The bounds SHALL be converted from their iCalendar UTC spelling to the RFC 3339 form the API takes.

### Requirement: gcal walks its own pagination
`list_items` SHALL follow `nextPageToken` until the requested window is covered, then apply the shared 1-indexed windowing. A page the caller never reaches SHALL NOT be fetched.

### Requirement: gcal refuses what Google cannot model
An item whose iCalendar carries no VEVENT SHALL be refused by component name rather than emulated, since Google models neither VTODO nor VJOURNAL. How the VEVENT itself projects is the [projection](./projection.md) capability's business.

### Requirement: vdir backend
vdir SHALL adapt io-vdir. A collection directory is a calendar and its metadata marker files carry the display name, description and color; each `.ics` file inside is an item, and a `.vcf` file is not. vdir has no entity tag, so `if_match` SHALL be ignored rather than refused. An update SHALL read the current metadata before writing, so a field the patch leaves untouched survives.

### Requirement: pimdir backend
pimdir SHALL adapt io-pimdir over io-replica. The store is an offline cache a sync engine fills, not a server: reads project the store's items and writes are staged io-replica mutations a later sync propagates.

Collections come from the sync, so `create_calendar`, `update_calendar` and `delete_calendar` SHALL refuse with a message pointing at the account the store syncs. A collection SHALL be listed as a calendar when it declares `text/calendar`, or when it declares no kind at all (a sync created it before any consumer declared one).

### Requirement: pimdir shows a short public id
The pimdir backend SHALL show and accept each item's public id (`items.seq`, a small store-assigned integer stable across every collection the item is filed in), not the internal `link_id`. It SHALL resolve that id to the `link_id` before reading a body or staging a change, and SHALL fail clearly on a non-numeric id rather than looking up nothing.

### Requirement: pimdir is an availability-aware cache
An item whose body is not local (`level < Full`, no stored object) SHALL still list, carrying no bytes. `get_item` on such an item SHALL report a clear "body not fetched" state, the cue to sync, not a data-loss error. A range filter SHALL still apply to it, read off the stored `text/calendar` summary rather than off bytes that are not local: a cache that hid its own undownloaded items from a date window would answer a different question than the one asked.

### Requirement: pimdir writes are staged and source-guarded
`create_item` SHALL stage an io-replica `Add`, `update_item` an `Edit` and `delete_item` a `Remove`, all through the store's `mutate` seam and never raw SQL. Each SHALL be attributed to the configured `pimdir.source`; on a store never synced as that source (the placement carries no base) the write SHALL fail loudly rather than stage a change no sync will carry. An `Edit` SHALL restate the sort key alongside the body, or an item whose DTSTART moved would stay sorted where its old start put it.

`create_item` SHALL content-hash the body with the same 128-bit FNV-1a digest as Neverest, himalaya and himalaya-android-m3, so an item calendula adds deduplicates against the same item a sync stored.

### Requirement: pimdir store path is shell-expanded
The pimdir backend SHALL expand `~` and environment variables on `pimdir.root` before opening the store and its blob reader. Opening the raw path would create an empty store at a literal `./~/…` relative to the working directory and silently return an empty calendar list.

### Requirement: pimdir writes auto-source
When `pimdir.source` is unset, the backend SHALL attribute its writes to the store's single synced source (via `distinct_sources`) when there is exactly one, which is the ordinary one-device case, falling back to `local` when the store has none or several.

### Requirement: The text/calendar summary convention
calendula SHALL write, and read, the pimdir `text/calendar` summary at `v: 1`: an optional `uid`, an optional `component` (`VEVENT`, `VTODO` or `VJOURNAL`), a required (possibly empty) `summary`, an optional `dtstart` carried verbatim beside its `dtstart_tzid` and `dtstart_value` (`date-time` or `date`), an optional verbatim `dtend`, an optional verbatim `due`, whether the item is `recurring`, and an optional `size`.

Times SHALL be carried verbatim rather than as resolved instants, so a reader with a time zone database re-derives an instant in its own zone and a reader without one displays the wall time the calendar wrote instead of a UTC claim the writer fabricated. The resolved instant SHALL NOT be duplicated in the summary: the `sort_key` is returned by the store's paging reads and is the single resolved projection.

The summary SHALL describe the master of a recurrence set, the component carrying no RECURRENCE-ID, since that is the item as a reader lists it. A resource carrying overrides alone SHALL still be summarised, from the first of them.

The companion `sort_key` SHALL hold the item's start normalised to RFC 3339 in UTC at seconds precision, read ascending: DTSTART for a VEVENT or a VJOURNAL, and DUE then DTSTART for a VTODO, which is scheduled by its due date (RFC 5545 3.8.2.3) and need not carry a DTSTART at all.

A UTC value SHALL be taken verbatim. A zoned one SHALL resolve through the VTIMEZONE the document carries, taking the earlier offset when the local time is ambiguous and the offset after the transition when it does not exist, since a local time at a transition names two instants or none. A zoned one whose zone the document does not define SHALL be read as floating rather than left unknown: the error is bounded by the offset and keeps the item near its place, where dropping the key would move every such item to the far end of the listing. A date-only value SHALL be read at midnight UTC and a floating one on the wall clock, both conventions rather than facts. An item with nothing parseable at all SHALL keep an empty key, which reads as unknown.

A recurring item SHALL key on its first occurrence, which is what DTSTART holds and is fixed for the life of the series. A date-range read over recurring items therefore needs the recurrence expanded above the store, expansion being a function of when you ask.
