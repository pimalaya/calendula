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
gcal SHALL adapt io-gcal's Calendar API v3 client. A calendar is a calendar list entry and an item is an event of it. Item ids SHALL be the event ids the API returned, verbatim. `create_item` SHALL use `events.import` when the projected event carries an iCalendar UID, so the UID survives, and `events.insert` otherwise. `update_item` SHALL honour `if_match` as the API's `If-Match` header, so the optimistic concurrency Google offers is not silently dropped.

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
calendula SHALL write, and read, the pimdir `text/calendar` summary at `v: 1`: an optional `uid`, a required (possibly empty) `summary`, optional `start` and `end` normalised to RFC 3339 in UTC at seconds precision, an optional dominant component `kind`, and an optional `size`. The companion `sort_key` SHALL hold DTSTART normalised the same way, so byte order is chronological order and a date-range read pages a calendar with the store's own statements. An item with no parseable start SHALL keep an empty key, which reads as unknown.
