---
cairn: spec
capability: backends
status: current
---

# Backends

Each backend is a `<Proto>Backend` adapter in `src/<proto>/backend.rs`, implementing the shared operations over one io-* crate's high-level client and converting its results into calendula's own shared types (`Calendar`, `CalendarDiff`, `CalendarItem`, `CalendarTimeRange`).

calendula owns those types; no aggregator library sits between it and the io-* crates.

### Requirement: Shared operation set
The shared adapters SHALL cover, per backend: `list_calendars`, `create_calendar`, `update_calendar`, `delete_calendar`, `list_items`, `get_item`, `create_item`, `update_item` and `delete_item`.

A backend that cannot model an operation SHALL refuse it with a message naming what to do instead, rather than emulating it.

### Requirement: Shared outputs carry no backend details
The output of a shared command SHALL NOT carry a field that only one backend fills. A backend detail worth showing SHALL be logged by that backend's adapter or shown by its own namespace.

### Requirement: A create reports the identifier it was given
`create_calendar` and `create_item` SHALL return the identifier the backend actually assigned, and the command SHALL report that one.

It is the requested id on every backend that lets a client name a collection, and a server-minted one where it does not, so a create never names a resource that does not exist.

### Requirement: Backend selection order
The shared commands SHALL target the backend the global `--backend` flag selects. Its default, `auto`, SHALL take the first configured-and-compiled backend in the order vdir, pimdir, CalDAV, gcal, msgraph, preferring a local read to a network round-trip and a protocol-standard server to a vendor API.

A named value SHALL pin the command to that backend and bail when the account carries no matching configuration block. The protocol-specific commands SHALL ignore the flag.

### Requirement: CalDAV backend
CalDAV SHALL adapt io-webdav's RFC 4791 surface over a connected client whose calendar home-set is already resolved.

Item ids SHALL be the resource names the server returned, verbatim: io-webdav neither appends nor strips a file extension, so an id a listing showed addresses the same resource on every verb.

A create SHALL propose a resource name derived from the item's own UID when that UID is URL-safe, falling back to a content digest, and SHALL keep the id the server reports in its `Location` header when it names the resource itself.

### Requirement: CalDAV pushes the time range down
A [`CalendarTimeRange`](#requirement-time-range-filtering) SHALL reach CalDAV as an RFC 4791 `time-range` filter nested in a VEVENT `comp-filter`, so the server does the narrowing.

The filter SHALL scope to VEVENT only, since RFC 4791 9.9 defines the overlap test against a component's own start and end, and a VTODO or VJOURNAL carrying neither would be dropped for the wrong reason.

### Requirement: Google Calendar backend
gcal SHALL adapt io-gcal's Calendar API v3 client. A calendar is a calendar list entry and an item is one calendar object resource of it: a single event, or a recurring event together with the exceptions that modify its instances (see [A recurrence set is one resource](./projection.md#requirement-a-recurrence-set-is-one-resource)).

Item ids SHALL be the event ids the API returned, verbatim, and the id of a set SHALL be the master's.

`create_item` SHALL use `events.import` when the projected event carries an iCalendar UID, so the UID survives, and `events.insert` otherwise. `update_item` SHALL honour `if_match` as the API's `If-Match` header, so the optimistic concurrency Google offers is not silently dropped.

`create_calendar` SHALL insert a secondary calendar. Google mints the id of a calendar it creates, so the requested id SHALL NOT be honoured and the [assigned one](#requirement-a-create-reports-the-identifier-it-was-given) SHALL be reported instead.

`update_calendar` SHALL patch the calendar for the name and the description and the calendar list entry for the colour, since Google splits those across the two resources. A Google calendar always carries a colour, so clearing one SHALL be refused by name.

### Requirement: gcal pushes the time range down
A [`CalendarTimeRange`](#requirement-time-range-filtering) SHALL reach gcal as the `timeMin` and `timeMax` parameters of `events.list`, so the server does the narrowing. The bounds SHALL be converted from their iCalendar UTC spelling to the RFC 3339 form the API takes.

### Requirement: gcal walks its own pagination
`list_items` SHALL follow `nextPageToken` until the requested window is covered, then apply the shared 1-indexed windowing. A page the caller never reaches SHALL NOT be fetched.

### Requirement: gcal refuses what Google cannot model
An item whose iCalendar carries no VEVENT SHALL be refused by component name rather than emulated, since Google models neither VTODO nor VJOURNAL. How the VEVENT itself projects is the [projection](./projection.md) capability's business.

### Requirement: Microsoft Graph backend
msgraph SHALL adapt io-msgraph's calendars and events. A calendar is a Graph calendar, its colour the read-only `hexColor`, and an item a lone event or a series master, keyed by the event id Graph returned, its `changeKey` as etag, its iCalendar contents synthesized by io-msgraph's `ical` projection with the exceptions of a series folded in, read from the instances of its own date range. Graph keeps no calendar description and names colours from a palette, so a create or update naming either SHALL be refused.

Graph narrows a listing by time only through the calendar view, which expands every series, so a range SHALL filter the listed items locally, as on vdir. Graph models a VEVENT and nothing else, so another kind SHALL be answered empty without a round-trip. Only the series master SHALL be written back: an exception edited here does not push, and an update naming an etag SHALL be refused when the master's `changeKey` moved.

### Requirement: vdir backend
vdir SHALL adapt io-vdir. A collection directory is a calendar and its metadata marker files carry the display name, description and color; each `.ics` file inside is an item, and a `.vcf` file is not.

vdir has no entity tag, so `if_match` SHALL be ignored rather than refused. An update SHALL read the current metadata before writing, so a field the patch leaves untouched survives.

### Requirement: pimdir backend
pimdir SHALL adapt io-pimdir, the store and the sync engine in one crate. The store is an offline cache a sync engine fills, not a server: reads project the store's items and writes are queue actions a later sync applies and propagates.

Collections come from the sync, so `create_calendar`, `update_calendar` and `delete_calendar` SHALL refuse with a message pointing at the account the store syncs.

A collection SHALL be listed as a calendar when it declares `text/calendar`, or when it declares no kind at all (a sync created it before any consumer declared one).

A store grouping its collections under accounts (pimdir SPEC 9.2) SHALL be narrowed to `pimdir.account` when that is set, and read whole when it is not.

### Requirement: pimdir takes the reader and producer roles, never the owner
The pimdir backend SHALL read through a `PimdirReader` and write through a `PimdirProducer`, and SHALL NOT open a `PimdirStore`.

The owner handle drains the queue, sweeps the objects and purges the trash, and holds an exclusive lock on the store for its lifetime, so holding it would both lock a sync out for the length of a listing and put every destructive verb behind a frontend that never calls them.

The reader SHALL be built with the pending overlay, so an action this process staged reads back before the store's owner applies it.

### Requirement: A pimdir calendar is its collection id
The pimdir backend SHALL show and accept a calendar as the store's collection id, verbatim: the collection `caldav/default` is the calendar `caldav/default`. It SHALL NOT derive, strip or accept a shortened spelling, and no configuration SHALL offer one.

A sync engine binds a source's collections under a namespace, so an id carries one. The store is opaque to it, neither parsing nor validating an id (pimdir SPEC 9.2) and modelling hierarchy through `parent` rather than through a separator.

Shortening is therefore a guess at the producer's convention, and one that makes a single calendar answer to two spellings.

### Requirement: pimdir refuses an unknown calendar
Every pimdir read and write SHALL fail when the calendar id names no collection of the account.

The store's read seam answers an unknown collection with an empty page and its queue accepts an action for any name, so without this a typo in `-k` would read as an empty calendar and stage into one nothing will ever apply.

The refusal SHALL name the calendars the account holds. Ids carry the sync engine's namespace and are not guessable, so an error asking for one that shows none leaves the user nothing to act on.

### Requirement: pimdir lists in the store's calendar order
A pimdir listing SHALL scan the collection by the store's own sort key, ascending, which is the item's resolved start. Paging by link id is an arbitrary order for a calendar, and the store maintains the one a reader expects.

### Requirement: pimdir shows a short public id
The pimdir backend SHALL show and accept each item's public id (`items.seq`, a small store-assigned integer stable across every collection the item is filed in), not the internal `link_id`.

It SHALL resolve that id to the `link_id` before reading a body or staging a change, and SHALL fail clearly on a non-numeric id rather than looking up nothing.

Addressing by the public id is what keeps two duplicated resources distinguishable: they carry one `UID` between them and have two `seq`s, so an address derived from the body would be ambiguous where a `seq` is not.

### Requirement: pimdir is an availability-aware cache
An item whose body is not local (`level < Full`, no stored object) SHALL still list, carrying no bytes. `get_item` on such an item SHALL report a clear "body not fetched" state, the cue to sync, not a data-loss error.

A range filter SHALL still apply to it, read off the stored `text/calendar` summary rather than off bytes that are not local: a cache that hid its own undownloaded items from a date window would answer a different question than the one asked.

An item naming an object whose blob file is absent is a different state: the store is inconsistent rather than partially synced. It SHALL still list, and the listing SHALL log a warning naming the item, since the row renders the same as an unhydrated one and `get_item` refuses it outright.

### Requirement: pimdir writes are staged queue actions
A pimdir write SHALL append one action to the store's queue (pimdir SPEC 15.1) through a producer opened for that write and dropped after it: `create_item` to `add`, `update_item` to `update`, `delete_item` to `remove`.

The body SHALL reach the blob tree through the blob writer, durably, before the row that pins it is appended, and the action SHALL address the item by the public `seq` that is already the item's shared id.

`update_item` SHALL ignore `--if-match`, because the engine reconciles the applied edit against the base body it recorded at sync time, which is stronger than an entity-tag precondition a local store cannot check.

Because a queued create carries no public id until the owner applies it, `create_item` SHALL report the item's link id instead.

### Requirement: pimdir queues scheduling intents
`pimdir reply <EVENT-ID> accept|tentative|decline` SHALL append one `calendar-reply` action `{ "v": 1, "source"?, "seq", "partstat": "ACCEPTED"|"TENTATIVE"|"DECLINED", "comment"? }`, and `pimdir cancel <EVENT-ID>` one `calendar-cancel` action `{ "v": 1, "source"?, "seq", "comment"? }` (pimdir STORAGE Annex B.2), anchored on the item's calendar, through the producer and its §15.6 gate, and SHALL touch no item.

`source` SHALL name the performer: `--source` when given, else the single candidate, else the recorded choice; several candidates and no choice SHALL be refused, listing them. A store whose sources declare nothing SHALL get no `source` unless one is given.

A reply to an event naming no `ORGANIZER`, or a cancellation of one naming no `ATTENDEE`, SHALL be refused when its body is local, the latter pointing at `event delete`.

### Requirement: A pimdir body is named by the store's own hash
The pimdir backend SHALL name a body it writes with the hash the store records in `store_meta.hash_algo`, read through the handle it holds, and SHALL NOT compute a digest of its own choosing. A body named under the wrong algorithm is a body no read ever finds.

### Requirement: pimdir store path is shell-expanded
The pimdir backend SHALL expand `~` and environment variables on `pimdir.root` before opening the store and its blob reader. Opening the raw path would create an empty store at a literal `./~/…` relative to the working directory and silently return an empty calendar list.

### Requirement: The text/calendar summary convention
A pimdir item's summary SHALL be the typed row of the store's event, task or journal table (pimdir STORAGE Annex A.3 to A.5), derived by `io_pimdir::summary::calendar`, the format's own derivation, so an item calendula stages links and summarises exactly as the same item arriving through a sync.

The link id is the bare `UID`, with nothing prepended, and the one derivation calendula runs: a queued create reports it, having no public id yet, and states it on the action so the reported id and the filed key agree by construction.

A queued action carries no summary and no sort key: the store's owner derives both from the body when it applies the action, and a producer deriving them would restate what the owner is about to derive.

calendula SHALL read that row to answer a date question about an item whose body is not local: `dtstart`, then `due` for a to-do carrying no start, the value verbatim so its leading day compares as a parsed `DTSTART` does. It SHALL read the row's table as the item's kind.

### Requirement: A UID is not an address
The pimdir backend SHALL NOT assume an item's link id is the `UID` its body carries, nor that a `UID` identifies at most one item in a calendar.

A store may hold two calendar object resources of one calendar sharing a `UID`, keyed apart by the store (pimdir SPEC 9), and both SHALL list, read and act as ordinary items, addressed by their own public `seq`.

What stays unique is the key and the public id: `(collection, link_id)` still names one item and `seq` still names one resource. What ends is the link id being derivable from the body, so a read that re-derives a `UID` in order to address a row is addressing an unknown number of them.

RFC 4791 4.1 requires the `UID` to be unique in the collection and servers do not always enforce it. The two copies need not even be the same event: a verified case held two different meetings under one `UID`. Resolving an identity to whichever row came first would hide one of them.

This is a second resource, not a second component: every component sharing a `UID` still lives in one resource, so each of the two items holds its own whole recurrence set.

### Requirement: The connection is opened by the call that needs it
`CalendarClient` SHALL select its backend from the account configuration without connecting, and open on the first call that needs the network. A command running a composer SHALL hold no connection open while the editor is up: a server closes an idle connection, and a write landing after a long edit would read the end of a socket nobody is on the other end of any more.

An `update -i`, which has to read the item before the editor, SHALL drop that connection before spawning the composer, the write afterwards opening a fresh one.
