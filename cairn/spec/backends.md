---
cairn: spec
capability: backends
status: current
---

# Backends

Each backend is a `<Proto>Backend` adapter in `src/<proto>/backend.rs`, implementing the unified operations over one io-* crate's high-level client and converting its results into calendula's own shared types (`Calendar`, `CalendarDiff`, `CalendarItem`, `CalendarTimeRange`).

calendula owns those types; no aggregator library sits between it and the io-* crates.

### Requirement: Unified operation set
The shared adapters SHALL cover, per backend: `list_calendars`, `create_calendar`, `update_calendar`, `delete_calendar`, `list_items`, `get_item`, `create_item`, `update_item` and `delete_item`.

A backend that cannot model an operation SHALL declare its capability `none` in its table, the detail naming what to do instead, rather than emulate it, and its adapter MAY refuse it again.

### Requirement: Unified outputs carry no backend details
The output of a unified command SHALL NOT carry a field that only one backend fills. A backend detail worth showing SHALL be logged by that backend's adapter or shown by its own namespace.

### Requirement: A create reports the identifier it was given
`create_calendar` and `create_item` SHALL return the identifier the backend actually assigned, and the command SHALL report that one.

It is the requested id on every backend that lets a client name a collection, and a server-minted one where it does not, so a create never names a resource that does not exist. A queued create has no identifier yet: it SHALL report the link id it staged, never a placeholder (no `q<id>`, `0` or empty string), and no command accepts that id until a sync applies the create.

### Requirement: A queued write is reported as queued
A backend whose writes are queued for a sync to push, rather than made on the server, SHALL have its writes confirmed as queued: `create` as "`<Kind>` `<id>` queued for the next sync", `update` as "`<Kind>` `<id>` update queued for the next sync", `delete` as "`<Kind>` deletion queued for the next sync", for the `event` and `item` families. Every other backend SHALL keep "successfully created", "updated", "deleted". The JSON payloads SHALL be the same for every backend.

pimdir is the only backend that queues its writes.

### Requirement: Backend selection order
The unified commands SHALL target the backend the global `--backend` flag selects. Its default, `auto`, SHALL take the first configured-and-compiled backend in the order vdir, pimdir, CalDAV, gcal, msgraph, preferring a local read to a network round-trip and a protocol-standard server to a vendor API.

`auto` SHALL NOT skip a backend for a capability it lacks: every unified verb exchanges calendar or item ids the others take, so all resolve to that one backend, and a refusal names the backend of the account serving the capability.

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

`create_item` SHALL use `events.insert` with `sendUpdates=all` when the projected event names at least one `ATTENDEE` and either no `ORGANIZER` or one whose address is the calendar id (case-insensitively), carrying its iCalendar UID as `iCalUID`, so the attendees are invited and the UID survives; a `--send-updates` given to the command SHALL replace that `all`. Otherwise it SHALL use `events.import` when the event carries a UID, so the UID survives, and `events.insert` when it carries none. `update_item` and `delete_item` SHALL honour `if_match` as the API's `If-Match` header, so the optimistic concurrency Google offers is not silently dropped. `update_item` and `delete_item` SHALL send `sendUpdates` only when the command was given `--send-updates`.

`create_calendar` SHALL insert a secondary calendar. Google mints the id of a calendar it creates, so the requested id SHALL NOT be honoured and the [assigned one](#requirement-a-create-reports-the-identifier-it-was-given) SHALL be reported instead.

`update_calendar` SHALL patch the calendar for the name and the description and the calendar list entry for the colour, since Google splits those across the two resources. A Google calendar always carries a colour, so clearing one SHALL be refused by name.

#### Scenario: An event created with attendees invites them
- GIVEN a gcal calendar and an event naming an `ATTENDEE` and no `ORGANIZER`, or the calendar's own address as `ORGANIZER`
- WHEN it is created
- THEN it is inserted with `sendUpdates=all` and its `iCalUID`, and Google mails the attendee

#### Scenario: Someone else's invitation is imported
- GIVEN a gcal calendar and an event whose `ORGANIZER` is another address
- WHEN it is created
- THEN it is imported, and nobody is mailed

### Requirement: gcal pushes the time range down
A [`CalendarTimeRange`](#requirement-time-range-filtering) SHALL reach gcal as the `timeMin` and `timeMax` parameters of `events.list`, so the server does the narrowing. The bounds SHALL be converted from their iCalendar UTC spelling to the RFC 3339 form the API takes.

### Requirement: gcal walks its own pagination
`list_items` SHALL follow `nextPageToken` until the requested window is covered, then apply the shared 1-indexed windowing. A page the caller never reaches SHALL NOT be fetched.

### Requirement: gcal refuses what Google cannot model
An item whose iCalendar carries no VEVENT SHALL be refused by component name rather than emulated, since Google models neither VTODO nor VJOURNAL. How the VEVENT itself projects is the [projection](./projection.md) capability's business.

### Requirement: Microsoft Graph backend
msgraph SHALL adapt io-msgraph's calendars and events. A calendar is a Graph calendar, its colour the read-only `hexColor`, and an item a lone event or a series master, keyed by the event id Graph returned, its `changeKey` as etag, its iCalendar contents synthesized by io-msgraph's `ical` projection with the exceptions of a series folded in, read from the instances of its own date range. Graph keeps no calendar description and names colours from a palette, so msgraph SHALL declare `calendar.description` and `calendar.color` `none`, a create or update naming either refused before connecting.

Graph narrows a listing by time only through the calendar view, which expands every series, so a range SHALL filter the listed items locally, as on vdir. Graph models a VEVENT and nothing else, so another kind SHALL be answered empty without a round-trip. Only the series master SHALL be written back: an exception edited here does not push, and an update naming an etag SHALL be refused when the master's `changeKey` moved.

### Requirement: vdir backend
vdir SHALL adapt io-vdir. A collection directory is a calendar and its metadata marker files carry the display name, description and color; each `.ics` file inside is an item, and a `.vcf` file is not.

vdir has no entity tag, so it SHALL declare `item.update.if-match` and `item.remove.if-match` `none`, `--if-match` refused before anything is read. An update SHALL read the current metadata before writing, so a field the patch leaves untouched survives.

### Requirement: pimdir backend
pimdir SHALL adapt io-pimdir, the store and the sync engine in one crate. The store is an offline cache a sync engine fills, not a server: reads project the store's items and writes are queue actions a later sync applies and propagates.

Collections come from the sync, so pimdir SHALL declare `calendar.create`, `calendar.rename`, `calendar.delete`, `calendar.description` and `calendar.color` `none`, a calendar write refused before the store is opened with a detail pointing at the account the store syncs; `create_calendar`, `update_calendar` and `delete_calendar` SHALL refuse it again.

A collection SHALL be listed as a calendar when it declares `text/calendar`, or when it declares no kind at all (a sync created it before any consumer declared one).

The backend SHALL read one account of the store (pimdir STORAGE §9.2): `pimdir.account` when set, else the one account every collection of the store is grouped under, or none when the store groups none. A store whose collections are grouped under several accounts, or partly ungrouped, SHALL be refused while `pimdir.account` is unset, the error naming the accounts and the key. Reads and writes SHALL be narrowed to that account's collections, and the producer bound to it.

### Requirement: pimdir takes the reader and producer roles, never the owner
The pimdir backend SHALL read through a `PimdirReader` and write through a `PimdirProducer`, and SHALL NOT open a `PimdirStore`.

The owner handle drains the queue, sweeps the objects and purges the trash, and holds an exclusive lock on the store for its lifetime, so holding it would both lock a sync out for the length of a listing and put every destructive verb behind a frontend that never calls them.

The reader SHALL be built with the pending overlay, so an action this process staged reads back before the store's owner applies it.

### Requirement: A pimdir calendar is its collection id
The pimdir backend SHALL show and accept a calendar as the store's collection id, verbatim: the collection `caldav/default` is the calendar `caldav/default`. It SHALL NOT derive, strip or accept a shortened spelling, and no configuration SHALL offer one.

A sync engine binds a source's collections under a namespace, so an id carries one. The store is opaque to it, neither parsing nor validating an id (pimdir STORAGE §9.2) and modelling hierarchy through `parent` rather than through a separator.

Shortening is therefore a guess at the producer's convention, and one that makes a single calendar answer to two spellings.

### Requirement: pimdir refuses an unknown calendar
Every pimdir read and write SHALL fail when the calendar id names no collection of the account.

The store's read seam answers an unknown collection with an empty page and its queue accepts an action for any name, so without this a typo in `-k` would read as an empty calendar and stage into one nothing will ever apply.

The refusal SHALL name the calendars the account holds. Ids carry the sync engine's namespace and are not guessable, so an error asking for one that shows none leaves the user nothing to act on.

### Requirement: pimdir lists in the store's calendar order
A pimdir listing SHALL scan the collection by the store's own sort key, ascending, which is the item's resolved start. Paging by link id is an arbitrary order for a calendar, and the store maintains the one a reader expects.

### Requirement: A pimdir listing streams the store
A pimdir listing SHALL walk the collection in keyset batches of the store's calendar order and hold one batch of rows, the body it is reading and the page it returns, never the collection. It SHALL stop reading once the page is full.

It SHALL read a body only to answer a kind or a window, or to return the item. Before the page of a listing naming neither, an item SHALL be counted off its row. An item whose stored summary sits in another component's table SHALL NOT be read for a kind, a resource holding one component type (pimdir STORAGE Annex A.3), and under a `UID` hint an item whose summary states another `UID` SHALL NOT be read, every component of a resource sharing one (RFC 4791 4.1); the summary keeps the value verbatim, so one holding an escape or padding SHALL be read rather than compared.

A summary SHALL NOT rule an item out by its dates: an override moves an occurrence anywhere and the summary describes the master alone. An item a pending `update` repoints SHALL NOT be ruled out by its summary, which describes the body it replaces.

### Requirement: pimdir shows a short public id
The pimdir backend SHALL show and accept each item's public id (`items.seq`, a small store-assigned integer stable across every collection the item is filed in), not the internal `link_id`.

It SHALL resolve that id to the `link_id` before reading a body or staging a change, and SHALL fail clearly on a non-numeric id rather than looking up nothing.

Addressing by the public id is what keeps two duplicated resources distinguishable: they carry one `UID` between them and have two `seq`s, so an address derived from the body would be ambiguous where a `seq` is not.

### Requirement: pimdir is an availability-aware cache
An item whose body is not local (`level < Full`, no stored object) SHALL still list, carrying no bytes. `get_item` on such an item SHALL report a clear "body not fetched" state, the cue to sync, not a data-loss error.

A range filter SHALL still apply to it, read off the stored `text/calendar` summary rather than off bytes that are not local: a cache that hid its own undownloaded items from a date window would answer a different question than the one asked.

An item naming an object the blob tree holds no body for is a different state: the store is inconsistent rather than partially synced. The blob file may be absent, or its entry no body at all (a directory, a link, bytes that do not hash to its name), which the store reads as absent (pimdir STORAGE §5, §7). It SHALL still list, and the listing SHALL log a warning naming the item, since the row renders the same as an unhydrated one and `get_item` refuses it outright.

### Requirement: pimdir writes are staged queue actions
A pimdir write SHALL append one action to the store's queue (pimdir STORAGE §15.1) through one producer opened for that write, before anything is written, and dropped after it: `create_item` to `add`, `update_item` to `update`, `delete_item` to `remove`.

A body SHALL be written with the row that pins it in one step, through that producer's `enqueue_with_body`, which hashes it under the store's algorithm, runs the store's gate once (pimdir STORAGE §15.6) and, for an `add`, the duplicate check on the bytes in hand, then writes the body durably and appends the row in one transaction under the producer's staging lock (§8): no collector can remove the body before its row pins it, and a refused write leaves no body in the tree. The partial supports the enqueue answers, those the gate passed the action with, the ones the body asks for (`calendar.scheduling`, `calendar.online-meeting`, `calendar.occurrence.update`) included, SHALL each be logged as a warning, for a `remove` and an intent too; calendula SHALL NOT run the gate apart from the enqueue. calendula SHALL NOT hash or write a body itself. The action SHALL address the item by the public `seq` that is already the item's shared id.

`update_item` and `delete_item` SHALL honour `if_match` against the item's version. The engine still reconciles an applied edit against the base body it recorded at sync time: the precondition is the caller's, the merge the store's.

Because a queued create carries no public id until the owner applies it, `create_item` SHALL report the item's link id instead.

### Requirement: pimdir refuses an online meeting no source can create
A `create_item` or `update_item` whose body newly asks for an online meeting (`X-PIMDIR-ONLINE-MEETING:TRUE`, an ask the item already carried being none) SHALL be refused, before its body is staged and anything is queued, unless a source concerned declares `calendar.online-meeting` fully or in part (STORAGE Annex B.1, §15.6): for a create, a source syncing the calendar; for an update, a source binding the item, the one that pushes it. A store whose sources declare nothing is refused too, since no sync engine there would act on the ask. The error SHALL start `Stage the pimdir action:` as the store's other refusals do, and name the capability and the sources declaring without it, or say no source of the store declares it. A source declaring it in part passes, with the store's partial-support warning.

### Requirement: pimdir refuses a to-do where its calendar takes none
A `create_item` or `update_item` whose body is a VTODO SHALL be refused, and nothing queued, when a source concerned states `calendar.component.vtodo` `none` for the calendar, or source-wide with no row of the calendar overriding it (pimdir STORAGE §15.6, Annex B): for a create, a source syncing the calendar; for an update, a source binding the item. The capability is read only where a row states it, so a source stating nothing of it, declared or not, refuses no to-do. The error SHALL start `Stage the pimdir action:`, say the calendar takes no to-do, name the source, the capability and its stated reason when it gives one, and point at `-k/--calendar` to store the to-do in another calendar.

### Requirement: pimdir refuses a create whose UID the calendar holds
A `create_item` whose link id (its `UID` under the store's derivation, pimdir STORAGE Annex A.3) names an item the calendar holds live, or a create already queued there, SHALL be refused through io-pimdir's `enqueue_with_body`, whose duplicate check, body write and append are one transaction, and nothing queued nor written. The error SHALL carry the code `item-duplicate`, name the calendar and the link id, and name the live item by its id, to be updated instead, or the queued create by its queue row.

### Requirement: account check takes both pimdir roles
`account check` on pimdir SHALL open the store as a reader, then open a producer and probe it (`PimdirProducer::probe`), which takes and releases the staging lock without waiting and the database's write lock, writing nothing into the store. A store a write cannot reach fails the check rather than the first write, with io-pimdir's code: a collector holding the staging lock fails it at once with `store-busy`, and the check leaves nothing in objects/.

### Requirement: A pimdir store failure carries io-pimdir's code
A failure to open the store, or to stage into it, SHALL carry the stable code io-pimdir gives it (`PimdirError::code()`) when it is one a caller acts on: `store-uncreated`, `store-version`, `store-foreign`, `store-corrupt`, `store-hash-algo`, `store-busy`, `store-exposed`, `lock-replaced`, `flag-invalid`, `action-malformed`. The message SHALL be io-pimdir's, but for `store-version`, and SHALL never advise deleting the store.

A store is never migrated (pimdir STORAGE §6): one of another format version than calendula's io-pimdir is refused as `store-version`, whichever way it differs. One naming no format version, or an earlier draft's, SHALL be refused with a message saying the store was written by an earlier sync engine, and giving the reinstall in the order that keeps its queued writes: sync it with the neverest that wrote it until nothing is queued, then reinstall it with `neverest sync --reset --force -a <account>`, naming the account neverest syncs it for, which moves it aside, deleting nothing, and syncs a new one. Any other SHALL be refused with a message saying a newer sync engine wrote it and to upgrade calendula.

### Requirement: A pimdir item's version is its body hash
The pimdir backend SHALL report as an item's `etag` the store's hash of its body, the pending queue folded in, so a staged update moves it at once. An item whose body is not local SHALL report none.

`update_item` and `delete_item` SHALL refuse, before anything is queued, a write whose `if_match` is not that version, surrounding double quotes ignored, with the code `precondition-failed` and an error starting `Precondition failed:`, naming the version found and the one expected. An item with no local body matches no version.

### Requirement: A queued pimdir edit names the body it replaced
An `update` that `update_item` queues, and a `remove` that `delete_item` queues, SHALL state as its `if_match` (pimdir STORAGE §15.3) the item's body hash when the write was gated on a version, by `--if-match` or by the read an update starting from the stored item makes, and none when the caller read no body. The store's owner then parks, rather than applies, an edit or a delete whose item holds another body by the time it drains, and the parked row SHALL show in `pimdir status` as every parked action does. A `remove` stating none deletes whatever the item holds.

#### Scenario: A sync lands first
- GIVEN an event read at version `A` and an `event update` queued from it
- WHEN a sync replaces the event's body with `B` before the queue drains
- THEN the owner parks the update, the event keeps `B`, and `pimdir status` lists the row with an error naming `A` and `B`

#### Scenario: A sync lands before a delete
- GIVEN an event at version `A` and an `event delete --if-match A` queued
- WHEN a sync replaces the event's body with `B` before the queue drains
- THEN the owner parks the remove, the event keeps `B`, and `pimdir status` lists the row with an error naming `A` and `B`

### Requirement: A pimdir write settles no conflict
An `update` and a `remove` calendula queues SHALL state no `resolves` (pimdir STORAGE §15.3): calendula shows no conflict and decides none. The store's owner parks, rather than applies, an edit or a delete meeting a conflict, the item's or that of the binding it runs through (§15.2), so the diverging body nobody read is neither replaced nor deleted, and the parked row SHALL show in `pimdir status` as every parked action does.

#### Scenario: A delete over a conflicted event
- GIVEN an event a sync left conflicted
- WHEN `event delete` queues its removal and the queue drains
- THEN the owner parks the remove, the event and its diverging body stay, and `pimdir status` lists the row with an error naming the conflict

### Requirement: pimdir queues scheduling intents
`pimdir reply <EVENT-ID> accept|tentative|decline` SHALL append one `calendar-reply` action `{ "v": 1, "source", "seq", "partstat": "ACCEPTED"|"TENTATIVE"|"DECLINED", "comment"?, "recurrence_id"? }`, and `pimdir cancel <EVENT-ID>` one `calendar-cancel` action `{ "v": 1, "source", "seq", "comment"?, "recurrence_id"? }` (pimdir STORAGE Annex B.2), built and validated by io-pimdir's `PimdirInvitation`, anchored on the item's calendar, through the producer and its §15.6 gate, and SHALL touch no item.

A payload its schema refuses SHALL be refused before anything is queued: a `partstat` outside the three, an item named by both or neither of `seq` and `link_id`, a cancel stating `partstat`, an empty `source`. calendula names the item by its `seq` alone, a reply's `partstat` from its answer and a cancel's none; a refusal io-pimdir raises carries `action-malformed`.

`source` SHALL name the performer in every payload. In a calendar a source syncing it declares capabilities for, it SHALL be `--source` when given, else the single candidate, else the recorded choice; several candidates and no choice SHALL be refused, listing them. In a calendar whose sources declare nothing, it SHALL be `--source` when given, else the one source syncing the calendar; several sources SHALL be refused, listing them, and none refused, both pointing at `--source`, nothing queued.

`-r/--recurrence-id` SHALL limit either to one occurrence, carried as `recurrence_id`, spelled as the `recurrenceId` an expanded listing shows (the series' `DTSTART` form). When the body is local, a value naming no occurrence of the series, or naming one in another spelling, SHALL be refused. A performer not declaring `calendar.reply.occurrence` or `calendar.cancel.occurrence`, and a store declaring nothing, SHALL be refused with the code `occurrence-unsupported` and an error starting `Occurrence not supported:`, naming the capability and the source, or saying no source of the store declares it; nothing is queued.

A reply to an event naming no `ORGANIZER`, or a cancellation of one naming no `ATTENDEE`, SHALL be refused when its body is local, the latter pointing at `event delete`. An event naming an object whose blob is absent SHALL refuse both, as `get_item` refuses its read, rather than queue them unchecked.

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

A store may hold two calendar object resources of one calendar sharing a `UID`, keyed apart by the store (pimdir STORAGE §9), and both SHALL list, read and act as ordinary items, addressed by their own public `seq`.

What stays unique is the key and the public id: `(collection, link_id)` still names one item and `seq` still names one resource. What ends is the link id being derivable from the body, so a read that re-derives a `UID` in order to address a row is addressing an unknown number of them.

RFC 4791 4.1 requires the `UID` to be unique in the collection and servers do not always enforce it. The two copies need not even be the same event: a verified case held two different meetings under one `UID`. Resolving an identity to whichever row came first would hide one of them.

This is a second resource, not a second component: every component sharing a `UID` still lives in one resource, so each of the two items holds its own whole recurrence set.

### Requirement: The connection is opened by the call that needs it
`CalendarClient` SHALL select its backend from the account configuration without connecting, and open on the first call that needs the network. A command running a composer SHALL hold no connection open while the editor is up: a server closes an idle connection, and a write landing after a long edit would read the end of a socket nobody is on the other end of any more.

An `update -i`, which has to read the item before the editor, SHALL drop that connection before spawning the composer, the write afterwards opening a fresh one.
