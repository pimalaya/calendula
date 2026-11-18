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

### Requirement: A server's refusal carries its code
A failure a server refused SHALL carry a code of the vocabulary Himalaya shares, read from the typed error of the protocol that answered it, the failure object, an `errors` entry and an `account check` row alike:

| Code | When |
|---|---|
| `permission-denied` | an HTTP 403 that is no rate limit (Google Calendar, Graph); a CalDAV 403 whose body names no precondition nor the iCalendar, or `need-privileges` (RFC 3744 §7.1.1) |
| `quota-exceeded` | an HTTP 507; a CalDAV body naming `quota-not-exceeded` (RFC 4331 §6); Graph's `ErrorQuotaExceeded` |
| `rate-limited` | an HTTP 429, or a Google Calendar 403 naming a rate limit or a quota; calendula retries nothing, so the first one met |
| `item-not-found` | an item request answered 404 or 410 (CalDAV, Google Calendar), a CalDAV item read answered 400 (iCloud's answer to a name it does not take, without `.ics`), a Google event read cancelled (a deleted event's stub), 404 or 400 `ErrorInvalidIdMalformed` (Graph); a vdir item file or a pimdir item that does not exist |
| `calendar-not-found` | a calendar named that does not exist (vdir, pimdir) |
| `calendar-duplicate` | a CalDAV MKCALENDAR naming `DAV:resource-must-be-null`, or answered 405 or 412 over a calendar the listing holds; Graph's `ErrorFolderExists`; a vdir directory the home holds (see "A calendar created twice is a duplicate") |
| `item-invalid` | a CalDAV write answered 400 or 403 whose body names a precondition on the item's content (RFC 4791 §5.3.2.1: `supported-calendar-data`, `valid-calendar-data`, `valid-calendar-object-resource`, `supported-calendar-component`) or the iCalendar itself (`icalendar`, `vcalendar`, case ignored), the predicate calendula's hint reads, so the two never disagree; a Google event write (`events.insert`, `events.import`, `events.update`) answered 400, calendula building every parameter of that request |
| `precondition-failed` | an HTTP 412 (CalDAV, Google Calendar, Graph), Graph's `--if-match` mismatch read before the write, a pimdir version that moved |
| `item-duplicate` | a CalDAV create refused for `no-uid-conflict` (RFC 4791 §5.3.2) or answered 412 to its `If-None-Match: *`, its resource name being derived from the UID; a gcal create whose UID the calendar holds; a pimdir create naming a `UID` its calendar holds or has queued |

A refusal none of these name SHALL carry no code, its reason kept in `sources`. A CalDAV write answered 400 naming nothing of the item SHALL carry none. Graph and vdir check no UID: a second create of one UID makes a second item there. A CalDAV 403 naming another precondition is an item the server would not take, no matter of rights, and SHALL NOT read `permission-denied`: `no-uid-conflict` is `item-duplicate`, a content precondition `item-invalid` under calendula's hint, the server's answer its cause, and any other (`max-resource-size`, `min-date-time`...) carries none. Graph's refusal of an event body carries none: its 400 names no body recognisably.

#### Scenario: An iCalendar the server refuses
- GIVEN a CalDAV server answering a `PUT` 403 with a `DAV:error` naming `C:valid-calendar-data`
- WHEN `calendula --json event add -k cal e.ics` runs
- THEN it exits 1 coded `item-invalid`, its error the hint saying calendula sends the item unchanged, the server's answer in `sources`

#### Scenario: A read-only Graph calendar
- GIVEN a Graph calendar the account may read but not write
- WHEN `calendula --json event delete <id>` runs
- THEN it exits 1 printing a failure object coded `permission-denied`

#### Scenario: An id iCloud does not take
- GIVEN a CalDAV server answering 400 to a GET of `<calendar>/nope`
- WHEN `calendula --json item read nope` or `event delete nope` runs
- THEN it exits 1 coded `item-not-found`, the server's answer in `sources`

### Requirement: Each backend names the items it could not read
A backend SHALL report an item it could not read as an `errors` entry and SHALL never drop it silently, with a warning alone, nor fail the listing for it:

- every backend: an item whose iCalendar does not parse SHALL be an `item-unreadable` entry in `event list`, `event search` and the agenda, no event row printed for it; `item list` SHALL list it as it lists any item, with no entry. A kind or a range filter applied after parsing SHALL keep such an item, whose kind and dates cannot be read;
- vdir: an item file that cannot be read SHALL be an `item-unreadable` entry naming the item, its error in `sources`; an item removed while it lists SHALL be left out with no entry;
- Graph: an event whose series master or instances the server refused while they were read again SHALL be an entry coded by the refusal (`item-unreadable` when the refusal carries no code), the other events listed; one answered 404 SHALL be left out with no entry;
- pimdir: a body that cannot be read SHALL be an `item-unreadable` entry, the item listed with no bytes; a hydrated body that does not parse SHALL be an entry where the kind filter dropped it; an item whose body is not local SHALL be no entry, whether it was never fetched or the item names a body its blob tree no longer holds (the blob file gone, or failing its name), which is not hydrated and not an error (pimdir GUIDE §14), as Himalaya and cardamum list such an item;
- CalDAV: a row the server answered without the item's data (refused whole, its `calendar-data` refused, or missing) SHALL be an entry naming the item, coded from its status, a 403 `permission-denied`, else `item-unreadable`, each refused property in `sources`; a listing the server truncated (a 507 row) SHALL be an entry with a `null` id and no code saying the listing is incomplete;
- CalDAV, Google Calendar: an answer or a page that fails SHALL fail the listing whole.

### Requirement: A calendar created twice is a duplicate
A `calendar create` naming a calendar that already exists SHALL fail coded `calendar-duplicate`, writing nothing, wherever the backend can tell (pimalaya-cli error.md, *A code names the domain's noun*): vdir, a directory of that name in the home, `vdir create` the same; CalDAV, a MKCALENDAR answered with a body naming `DAV:resource-must-be-null` (RFC 4791 §5.3.1.1), or answered 405 or 412 when a listing then holds the calendar, the server's answer in `sources`; Graph, `ErrorFolderExists`, Exchange keeping one calendar per name. Google Calendar mints the id and lets two calendars share a name, so it holds no duplicate to refuse; pimdir serves no calendar create. Every path SHALL word it ``Cannot create calendar `<calendar>`: it exists already``, `<calendar>` the id on vdir and CalDAV and the name on Graph, the server's answer, when one refused it, in `sources`.

#### Scenario: A vdir calendar created twice
- GIVEN a vdir home holding the calendar `work`, named `Work`
- WHEN `calendula --json calendar create Other --id work` runs, then `calendula --json vdir create work -n Other`
- THEN each exits 1 coded `calendar-duplicate`, ``Cannot create calendar `work`: it exists already``, and `work` is still named `Work`

### Requirement: Backend selection order
The unified commands SHALL target the backend the global `--backend` flag selects. Its default, `auto`, SHALL take the first configured backend in the order vdir, pimdir, CalDAV, gcal, msgraph, preferring a local read to a network round-trip and a protocol-standard server to a vendor API.

When that backend is one the build leaves out, its block loaded unread (config, "Unknown fields"), `auto` SHALL NOT fall through to the next: the command SHALL fail with `backend-not-built` before anything is read or connected, the message naming the backend and the cargo feature bringing it back, `--features gcal` for gcal, and, when the account configures a backend the build serves, the first of them and the `--backend` reaching it. One configuration therefore reaches the same backend in every build, or fails saying why. A `--backend` value naming a backend the build leaves out SHALL be refused by the command line (exit 2), naming the cargo feature.

`auto` SHALL NOT skip a backend for a capability it lacks: every unified verb exchanges calendar or item ids the others take, so all resolve to that one backend, and a refusal names the backend of the account serving the capability.

A named value SHALL pin the command to that backend and bail when the account carries no matching configuration block. The protocol-specific commands SHALL ignore the flag.

#### Scenario: The first backend is left out of the build
- GIVEN a build with `msgraph` and without `gcal`, and an account carrying a `gcal` and a `msgraph` block
- WHEN `calendula --json calendar list` runs
- THEN it exits 1 with the failure object coded `backend-not-built`, naming `--features gcal` and `--backend msgraph`, and nothing is connected

### Requirement: CalDAV backend
CalDAV SHALL adapt io-webdav's RFC 4791 surface over a connected client whose calendar home-set is already resolved.

The client SHALL connect from `caldav.server` or `caldav.home` alone ("CalDAV locates its home-set two ways" in config), and SHALL NOT resolve a domain through DNS (RFC 6764 SRV records), so the backend links no resolver.

Calendar and item ids SHALL be the last segments of the hrefs the server serves, read as io-webdav reads a listed segment: percent-decoded when the decoding encodes back to that very segment, as served otherwise (a `%2F`, an escape of a byte sent raw such as `%40`, lowercase hex). A calendar outside the home set SHALL be named by its path, as served. io-webdav neither appends nor strips a file extension, so an id a listing showed addresses the same resource on every verb, as does the served segment itself. A calendar a command names SHALL be found among the listed ones by the path its id addresses under the home set (`rfc4918::collection_path`), never by comparing the strings, so either spelling finds it and a segment kept as served is never found by its decoding.

A create SHALL propose a resource name derived from the UID of the item's first VEVENT, VTODO or VJOURNAL when that UID is URL-safe, falling back to a content digest, and SHALL keep the id the server reports in its `Location` header when it names the resource itself, else the last segment of the URL io-webdav sent, both read as a listed segment is, which is the form a listing returns.

A listing the server answers with anything but one whole multistatus (an HTML page, a cut document, an empty body) SHALL fail, never read as a shorter listing, and a delete answered by a multistatus naming failed members SHALL fail.

A listing SHALL name, in its `errors`, each row the server answered without the item's data and a listing the server truncated ("Each backend names the items it could not read"), on the first page only: the whole calendar is read before the page is cut, and such a row holds no place on a page.

`update_item` given no version SHALL send `If-Match: *` (RFC 9110 §13.1.1), so an id the calendar does not hold is refused, its 412 coded `item-not-found`, rather than created and reported as an update.

A calendar's colour SHALL be written as Apple's `calendar-color` (`http://apple.com/ns/ical/`), by a create and an update alike, and read from it, the inf-it one when it is absent.

`update_calendar` SHALL write only the properties the patch names, setting a given one and removing (`DAV:remove`) a cleared one, through io-webdav's calendar update (`CaldavCalendarPatch`), which sends a request again on a fresh connection when the server closed the one it was sent on: a property the server refuses, or answers nothing of (RFC 4918 §9.2.1), fails the update. A colour SHALL go first, in a PROPPATCH of its own, so a server keeping the rest of a PROPPATCH while it ignores a colour leaves the calendar as it was: the update fails ``Write the colour of calendar `<id>`, nothing else written``. A calendar no listed id addresses SHALL fail `calendar-not-found` before anything is written. `create_calendar` given a colour SHALL read the calendars back and log a warning naming the colour when the server kept none.

#### Scenario: A colour the server ignores
- GIVEN a CalDAV server ignoring `calendar-color` in a PROPPATCH
- WHEN `calendula calendar update cal -n New -d Text -C '#123456'` runs
- THEN it exits 1 naming `calendar-color`, after one PROPPATCH holding the colour alone, and the name and the description are not written

#### Scenario: A calendar named with a space
- GIVEN a CalDAV calendar the server serves at `<home>/My%20Cal/`
- WHEN `calendula calendar list` runs, then `calendula calendar update "My Cal" -n New` and `calendula calendar update My%20Cal -n New`
- THEN the calendar lists as `My Cal`, and each update sends its PROPPATCH to `<home>/My%20Cal/`

### Requirement: account check authenticates on a pinned CalDAV home
`account check` on CalDAV SHALL send at least one authenticated request: on `server`, the principal and home-set discovery the client connects with; on `home`, which skips discovery, a PROPFIND Depth 0 on the pinned home-set asking its `DAV:resourcetype`. That PROPFIND SHALL be the check's alone: a unified command connecting on `home` sends nothing before its own first request, which a refused credential fails. A credential the server refuses SHALL fail the row on either route with the same code and error (`WebDAV server returned HTTP 401`, no code); a home answering anything but a multistatus fails it too.

#### Scenario: A refused credential on a pinned home
- GIVEN the account `test` whose `caldav.home` pins a home-set on a server answering 401 to its credential
- WHEN `calendula --json account check` runs
- THEN the `caldav` row reads `"ok": false`, `"code": null` and the error the same account gives on `caldav.server`, and the command exits 3

### Requirement: A CalDAV request survives a closed connection
Every CalDAV request calendula sends, a unified command's, a `caldav` command's (`report raw` included) and `account check`'s PROPFIND on a pinned home alike, SHALL go through io-webdav's client, by one of its methods or, for a request it has no method for, `WebdavClientStd::run`, never by hand over its stream: a request meeting a connection the server closed before answering it (`Connection: close`, an idle keep-alive timing out) is sent again, once, on a fresh connection to the same server. A refusal such a request meets SHALL carry the code any CalDAV refusal carries ("A server's refusal carries its code").

#### Scenario: A pinned home closing the check's connection
- GIVEN the account `test` whose `caldav.home` pins a home-set on a server closing the check's first connection unanswered, then answering its PROPFIND
- WHEN `calendula --json account check` runs
- THEN the PROPFIND is sent again on a fresh connection, the `caldav` row reads `"ok": true` and the command exits 0

#### Scenario: A raw REPORT refused
- GIVEN a CalDAV server answering a REPORT 403 with no precondition
- WHEN `calendula --json caldav report raw cal query.xml` runs
- THEN it exits 1 coded `permission-denied`

### Requirement: CalDAV pushes the time range down
A [`CalendarTimeRange`](#requirement-time-range-filtering) SHALL reach CalDAV as an RFC 4791 `time-range` filter nested in a VEVENT `comp-filter`, so the server does the narrowing.

The filter SHALL scope to VEVENT only, since RFC 4791 9.9 defines the overlap test against a component's own start and end, and a VTODO or VJOURNAL carrying neither would be dropped for the wrong reason.

### Requirement: A CalDAV id addresses its own resource
A calendar or item id a CalDAV command names SHALL be refused, before the account's secret is resolved or anything connects, when io-webdav refuses it (`rfc4918::check_collection_id`, `check_resource_id`: empty, or holding an empty, `.` or `..` segment once split on `/` and `\` and percent-decoded, a calendar id's trailing slashes and an absolute id's leading one aside), such an id addressing the home set, its parent or the server root: a calendar id coded `calendar-not-found`, ``Calendar `<id>` not found``, an item id coded `item-not-found`, ``Item `<id>` not found``, io-webdav's reason in `sources`, as vdir refuses an id that is not one directory name. Every unified command naming one SHALL refuse it so (`calendar update` and `delete`, `event list`, `agenda`, `search`, `read`, `add`, `update` and `delete`, `item list`, `read`, `add`, `update` and `delete`), `event delete` in its own sentence for an item id (commands, "An event is named by its id alone"), and every `caldav` command naming one (`propfind`, `proppatch`, `report`, `get`, `put` and `delete`). The id `calendar create` or `caldav mkcalendar` gives a new calendar, its URL segment, SHALL be refused the same way, uncoded as vdir refuses a new id: ``Calendar id `<id>` names no CalDAV collection``. A name given alone takes a minted UUID.

io-webdav refuses such an id itself before sending anything, but as an I/O error no code reads, and only once the account is open.

#### Scenario: An id climbing out of the home set
- GIVEN a CalDAV account whose password command would leave a file behind
- WHEN `calendula --json calendar delete ..` or `calendula --json item read -k cal ..` runs
- THEN it exits 1 coded `calendar-not-found` (``Calendar `..` not found``) or `item-not-found` (``Item `..` not found``), io-webdav's reason in `sources`, the password command never ran and nothing connected

### Requirement: A CalDAV item reads whole
An item a CalDAV listing (`event list`, `item list`, `event search`, `event agenda`) reads SHALL carry the whole `calendar-data` text, the final CRLF a server escapes as `&#13;` included, byte-identical to the body `item read` returns.

### Requirement: Google Calendar backend
gcal SHALL adapt io-gcal's Calendar API v3 client. A calendar is a calendar list entry and an item is one calendar object resource of it: a single event, or a recurring event together with the exceptions that modify its instances (see [A recurrence set is one resource](./projection.md#requirement-a-recurrence-set-is-one-resource)).

Item ids SHALL be the event ids the API returned, verbatim, and the id of a set SHALL be the master's. A calendar, event or rule id SHALL reach io-gcal as it stands, never percent-encoded by calendula: the crate encodes each as one path segment, so an id holding `#`, `?` or `/` names its own resource, and one encoded twice would name another.

`create_item` SHALL use `events.insert` with `sendUpdates=all` when the projected event names at least one `ATTENDEE` and either no `ORGANIZER` or one whose address is the calendar id (case-insensitively), carrying its iCalendar UID as `iCalUID`, so the attendees are invited and the UID survives; a `--send-updates` given to the command SHALL replace that `all`. Otherwise it SHALL use `events.import` when the event carries a UID, so the UID survives, and `events.insert` when it carries none. `update_item` and `delete_item` SHALL honour `if_match` as the API's `If-Match` header, so the optimistic concurrency Google offers is not silently dropped. `update_item` and `delete_item` SHALL send `sendUpdates` only when the command was given `--send-updates`.

A write of a series SHALL go through io-gcal's `GcalEventSeries`: the master written with its EXDATEs among its recurrence lines, then each override merged onto the instance `events.instances` lists for it within the override's `instance_window` (the one it `replaces`) and updated with that instance's etag, mailing as the master's write mails (an import mails nobody). An override no listing holds SHALL be skipped when it is cancelled, refused otherwise. A create whose override fails SHALL delete the series again. What Google cannot hold (`left_out`) SHALL be left out with a warning naming it. A zoned boundary SHALL project in the zone Google names (`DTSTART;TZID=<zone>`), so a series keeps its wall time across a daylight-saving change (io-gcal 0.5.1).

Before it inserts or imports an event carrying a UID, `create_item` SHALL look the UID up (`events.list` filtered on the `iCalUID`, every page until one holds a match, a deleted event not listed) and SHALL refuse a UID the calendar holds with `item-duplicate`, ``Calendar `<cal>` already holds `<uid>` as event `<id>`; update that event instead``, `<id>` the series master's, sending no write: an import matches on the `iCalUID` and would overwrite that event.

A read of an event Google holds as cancelled, a deleted event's stub, SHALL be `item-not-found` (`event read`, `item read`, the base an update reads), as a 404 is.

`create_calendar` SHALL insert a secondary calendar. Google mints the id of a calendar it creates, so gcal SHALL declare `calendar.create.id` `none`, `--id` refused before connecting, and the [assigned one](#requirement-a-create-reports-the-identifier-it-was-given) SHALL be reported.

`update_calendar` SHALL patch the calendar for the name and the description and the calendar list entry for the colour, since Google splits those across the two resources. A cleared description SHALL be sent as an empty string, a field a PATCH leaves out staying as it is, and an empty description SHALL read as none. A Google calendar always carries a colour, so clearing one SHALL be refused by name, before any request. `create_calendar` given a colour SHALL insert the calendar, then paint its calendar list entry; a failed paint SHALL name the calendar created and say to set the colour with `calendar update`, so a retry does not create a second one. The user's primary calendar (`primary` on its calendar list entry) SHALL be the `default` of `calendar list`.

#### Scenario: An event created with attendees invites them
- GIVEN a gcal calendar and an event naming an `ATTENDEE` and no `ORGANIZER`, or the calendar's own address as `ORGANIZER`
- WHEN it is created
- THEN it is inserted with `sendUpdates=all` and its `iCalUID`, and Google mails the attendee

#### Scenario: Someone else's invitation is imported
- GIVEN a gcal calendar and an event whose `ORGANIZER` is another address
- WHEN it is created
- THEN it is imported, and nobody is mailed

#### Scenario: A UID the calendar holds
- GIVEN a gcal calendar holding an event of UID `e1@example.org`
- WHEN `calendula --json event add` of another event carrying that UID runs
- THEN it exits 1 coded `item-duplicate` naming the held event, and neither `events.insert` nor `events.import` is sent

### Requirement: gcal pushes the time range down
A [`CalendarTimeRange`](#requirement-time-range-filtering) SHALL reach gcal as the `timeMin` and `timeMax` parameters of `events.list`, so the server does the narrowing. The bounds SHALL be converted from their iCalendar UTC spelling to the RFC 3339 form the API takes.

### Requirement: gcal walks its own pagination
`list_items` SHALL follow `nextPageToken` until the requested window is covered, then apply the shared 1-indexed windowing. A page the caller never reaches SHALL NOT be fetched.

### Requirement: gcal refuses what Google cannot model
An item whose iCalendar carries no VEVENT SHALL be refused by component name rather than emulated, since Google models neither VTODO nor VJOURNAL. How the VEVENT itself projects is the [projection](./projection.md) capability's business.

### Requirement: Microsoft Graph backend
msgraph SHALL adapt io-msgraph's calendars and events. A calendar is a Graph calendar, its colour the read-only `hexColor`, and an item a lone event or a series master, keyed by the event id Graph returned, its `changeKey` as etag, its iCalendar contents synthesized by io-msgraph's `ical` projection with the exceptions of a series folded in. The exceptions SHALL be read from the series' instances within the range the command lists, a day wider either side for the zones, or within five years either side of today when it lists none, cut by io-msgraph's `instance_windows` into windows Graph accepts: no `instances` request SHALL span five years or more. A window Graph refuses SHALL make its event an item error, never fail the listing. An update SHALL read its base over the range a read of the item does. Graph keeps no calendar description and names colours from a palette, so msgraph SHALL declare `calendar.description` and `calendar.color` `none`, a create or update naming either refused before connecting. An empty `hexColor`, Graph's spelling of no colour, SHALL read as none, and the calendar Graph marks `isDefaultCalendar` SHALL be the `default` of `calendar list`.

A listing SHALL follow its pages through io-msgraph, which refuses a paging link leaving the Graph origin, the request it builds carrying the account's token.

Graph narrows a listing by time only through the calendar view, which expands every series, so a range SHALL filter the listed items locally, as on vdir. A series master whose recurrence range, as the listing gives it, ends before the range or starts after it SHALL be left out unread, neither read again for its cancelled occurrences nor its instances listed. Graph models a VEVENT and nothing else, so another kind SHALL be answered empty without a round-trip. A write of a series SHALL go through io-msgraph's `MsgraphEventSeries`: the master created or patched, then each occurrence the document cancels (an EXDATE) deleted and each it overrides patched, through the instance `event_instances` lists for it in the change's window (`is_instance`); a delete whose instance is not listed is done already, a patch naming an occurrence the series does not have SHALL be refused. A create whose occurrence fails SHALL delete the series again. What Graph cannot hold (`left_out`: an RDATE, an EXRULE, a RANGE) SHALL be left out with a warning naming it. An update naming an etag SHALL be refused when the master's `changeKey` moved. Graph mints a calendar's id, so msgraph SHALL declare `calendar.create.id` `none`.

#### Scenario: An open-ended series older than five years
- GIVEN a Microsoft 365 calendar holding a weekly series started six years ago with no end, one occurrence this week moved by a day
- WHEN `event list` lists this week
- THEN it succeeds, and lists the moved occurrence on its new day

#### Scenario: A calendar holding years of ended series
- GIVEN a Microsoft 365 calendar holding 135 series masters, 117 of them ended before this week
- WHEN `event list --from` and `--to` list this week
- THEN no request reads one of the 117 again or lists its instances

### Requirement: vdir backend
vdir SHALL adapt io-vdir. A collection directory is a calendar and its metadata marker files carry the display name, description and color; each `.ics` file inside is an item, and a `.vcf` file is not.

vdir has no entity tag, so it SHALL declare `item.update.if-match` and `item.remove.if-match` `none`, `--if-match` refused before anything is read. An update SHALL read the current metadata before writing, so a field the patch leaves untouched survives.

### Requirement: vdir refuses an unknown calendar
A vdir calendar id SHALL name a calendar only when it is one directory name, one path segment that is neither `.` nor `..` and holds no separator (a trailing one included), naming a directory under the home. Every vdir command naming another SHALL fail before anything is read or written, coded `calendar-not-found`: `calendar update`, `calendar delete` and every `event` and `item` read and write with ``Calendar `<id>` not found``, `vdir delete` and `vdir rename` with ``Collection `<id>` not found``. A new id that is not one directory name SHALL be refused before anything is written: `calendar create` with ``Calendar id `<id>` is not one directory name``, or, given a name and no id, ``Calendar name `<name>` is not one directory name: give the calendar an id with `--id` ``, and `vdir create` with ``Collection id `<id>` is not one directory name``, uncoded, the new name of `vdir rename` by io-vdir.

A write creates whatever directory it is handed, a listing of a missing one answers nothing and a delete removes whatever the path reaches, so without it a typo in `-k` would read as an empty calendar or invent one, an id reaching out of the home (`..`, `../keep`, `personal/..`) would remove a directory there and a create would write there.

#### Scenario: A typo in `-k`
- GIVEN the account `test` whose vdir home holds the calendar `personal` alone
- WHEN `calendula --json event list -k nope`, `event add -k nope` or `calendar delete -k ..` runs
- THEN each exits 1 with the failure object coded `calendar-not-found`, and the home and the directory holding it are left as they were

#### Scenario: A new id reaching out of the home
- GIVEN the account `test` whose vdir home `outer/home` holds the calendar `personal` alone, beside the directory `outer/keep`
- WHEN `calendula --json calendar create Escape --id ..`, `calendar create Escape --id ../keep` or `vdir create x/y` runs
- THEN each exits 1 with the failure object, its `code` null, and nothing is written in the home, `outer/keep` or `outer`

### Requirement: pimdir backend
pimdir SHALL adapt io-pimdir, the store and the sync engine in one crate. The store is an offline cache a sync engine fills, not a server: reads project the store's items and writes are queue actions a later sync applies and propagates.

Collections come from the sync, so pimdir SHALL declare `calendar.create`, `calendar.create.id`, `calendar.rename`, `calendar.delete`, `calendar.description` and `calendar.color` `none`, a calendar write refused before the store is opened with a detail pointing at the account the store syncs; `create_calendar`, `update_calendar` and `delete_calendar` SHALL refuse it again.

A collection SHALL be listed as a calendar when it declares `text/calendar`, or when it declares no kind at all (a sync created it before any consumer declared one).

The backend SHALL read one account of the store (pimdir STORAGE §9.2): `pimdir.account` when set, else the one account every collection of the store is grouped under, or none when the store groups none. A store whose collections are grouped under several accounts, or partly ungrouped, SHALL be refused while `pimdir.account` is unset, the error naming the accounts and the key. Reads and writes SHALL be narrowed to that account's collections, and the producer bound to it.

### Requirement: pimdir takes the reader and producer roles, the owner for one cancel
The pimdir backend SHALL read through a `PimdirReader` and write through a `PimdirProducer`, and SHALL NOT open a `PimdirStore`, but in `pimdir queue cancel`, which drops one queue row as Himalaya's does: it opens the store as its owner for that call alone, through `PimdirStore::open_current`, which creates nothing, and fails `store-owned` at once while a sync holds it.

The owner handle drains the queue, sweeps the objects and purges the trash, and holds an exclusive lock on the store for its lifetime, so holding it would both lock a sync out for the length of a listing and put every destructive verb behind a frontend that never calls them. Dropping a row is an owner write no producer can make, and the cancel holds the lock for that write alone.

The reader SHALL be built with the pending overlay, so an action this process staged reads back before the store's owner applies it.

### Requirement: A pimdir calendar is its collection id
The pimdir backend SHALL show and accept a calendar as the store's collection id, verbatim: the collection `caldav/default` is the calendar `caldav/default`. It SHALL NOT derive, strip or accept a shortened spelling, and no configuration SHALL offer one.

A sync engine binds a source's collections under a namespace, so an id carries one. The store is opaque to it, neither parsing nor validating an id (pimdir STORAGE §9.2) and modelling hierarchy through `parent` rather than through a separator.

Shortening is therefore a guess at the producer's convention, and one that makes a single calendar answer to two spellings.

### Requirement: pimdir refuses an unknown calendar
Every pimdir read and write SHALL fail when the calendar id names no collection of the account.

The store's read seam answers an unknown collection with an empty page and its queue accepts an action for any name, so without this a typo in `-k` would read as an empty calendar and stage into one nothing will ever apply.

The refusal SHALL carry the code `calendar-not-found` and name the calendars the account holds. Ids carry the sync engine's namespace and are not guessable, so an error asking for one that shows none leaves the user nothing to act on.

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
An item whose body is not local SHALL still list, carrying no bytes and no `etag`: one never fetched (`level < Full`, no stored object), and one naming an object the blob tree holds no body for, the blob file absent or its entry no body at all (a directory, a link, bytes that do not hash to its name), which the store reads as absent (pimdir STORAGE §5, §7) and which is not hydrated and not an error (pimdir GUIDE §14). `get_item` on such an item SHALL fail coded `body-pending`, ``Item `<id>` in calendar `<calendar>` is not downloaded yet (body not fetched); run a sync to hydrate it``, the cue to sync, not a data-loss error, exit 1, as Himalaya reads such a message.

A range filter SHALL still apply to it, read off the stored `text/calendar` summary rather than off bytes that are not local: a cache that hid its own undownloaded items from a date window would answer a different question than the one asked.

A body the store fails to read (an I/O error) is a different state: the item SHALL still list, carrying no bytes, and the listing SHALL name it in its `errors`, coded `item-unreadable`, since the row renders the same as an unhydrated one.

#### Scenario: A body gone from the blob tree
- GIVEN a pimdir calendar holding a synced event whose blob file was removed
- WHEN `calendula --json item list` runs
- THEN it lists the item with no bytes and `etag` `null`, `errors` empty, exit 0
- AND `calendula --json item read <id>` fails coded `body-pending`, exit 1

### Requirement: pimdir writes are staged queue actions
A pimdir write SHALL append one action to the store's queue (pimdir STORAGE §15.1) through one producer opened for that write, before anything is written, and dropped after it: `create_item` to `add`, `update_item` to `update`, `delete_item` to `remove`.

A body SHALL be written with the row that pins it in one step, through that producer's `enqueue_with_body`, which hashes it under the store's algorithm, runs the store's gate once (pimdir STORAGE §15.6) and, for an `add`, the duplicate check on the bytes in hand, then writes the body durably and appends the row in one transaction under the producer's staging lock (§8): no collector can remove the body before its row pins it, and a refused write leaves no body in the tree. The partial supports the enqueue answers, those the gate passed the action with, the ones the body asks for (`calendar.scheduling`, `calendar.online-meeting`, `calendar.occurrence.update`) included, SHALL each be logged as a warning, for a `remove` and an intent too; calendula SHALL NOT run the gate apart from the enqueue. calendula SHALL NOT hash or write a body itself. The action SHALL address the item by the public `seq` that is already the item's shared id.

`update_item` and `delete_item` SHALL honour `if_match` against the item's version. The engine still reconciles an applied edit against the base body it recorded at sync time: the precondition is the caller's, the merge the store's.

Because a queued create carries no public id until the owner applies it, `create_item` SHALL report the item's link id instead.

### Requirement: pimdir refuses an online meeting no source can create
A `create_item` or `update_item` whose body newly asks for an online meeting (`X-PIMDIR-ONLINE-MEETING:TRUE`, an ask the item already carried being none) SHALL be refused, before its body is staged and anything is queued, unless a source concerned declares `calendar.online-meeting` fully or in part (STORAGE Annex B.1, §15.6): for a create, a source syncing the calendar; for an update, a source binding the item, the one that pushes it. A store whose sources declare nothing is refused too, since no sync engine there would act on the ask. The error SHALL start `Stage the pimdir action:` as the store's other refusals do, and name the capability and the sources declaring without it, or say no source of the store declares it. A source declaring it in part passes, with the store's partial-support warning. The raw `item add` queues an imported file whole, so an `X-PIMDIR-` property the file carries is honoured as the user's ask: a known departure from pimdir STORAGE §18, which has a producer remove every such property from a resource the user did not write.

### Requirement: pimdir refuses a to-do where its calendar takes none
A `create_item` or `update_item` whose body is a VTODO SHALL be refused, and nothing queued, when a source concerned states `calendar.component.vtodo` `none` for the calendar, or source-wide with no row of the calendar overriding it (pimdir STORAGE §15.6, Annex B): for a create, a source syncing the calendar; for an update, a source binding the item. The capability is read only where a row states it, so a source stating nothing of it, declared or not, refuses no to-do. The error SHALL start `Stage the pimdir action:`, say the calendar takes no to-do, name the source, the capability and its stated reason when it gives one, and point at `-k/--calendar` to store the to-do in another calendar.

### Requirement: pimdir refuses a create whose UID the calendar holds
A `create_item` whose link id (its `UID` under the store's derivation, pimdir STORAGE Annex A.3) names an item the calendar holds live, or a create already queued there, SHALL be refused through io-pimdir's `enqueue_with_body`, whose duplicate check, body write and append are one transaction, and nothing queued nor written. The error SHALL carry the code `item-duplicate`, name the calendar and the link id, and name the live item by its id, to be updated instead, or the queued create by its queue row.

### Requirement: account check takes both pimdir roles
`account check` on pimdir SHALL open the store as a reader, then open a producer and probe it (`PimdirProducer::probe`), which takes and releases the staging lock without waiting and the database's write lock, writing nothing into the store. A store a write cannot reach fails the check rather than the first write, with io-pimdir's code: a collector holding the staging lock fails it at once with `store-busy`, and the check leaves nothing in objects/.

### Requirement: A pimdir store failure carries io-pimdir's code
A store failure SHALL carry the stable code io-pimdir gives it (`PimdirError::code()`) when it is one Himalaya relays, by the same spelling: `store-version`, `store-uncreated`, `store-foreign`, `store-corrupt`, `store-not-current`, `store-hash-algo`, `sync-version`, `store-busy`, `store-owned`, `store-staging`, `store-in-flight`, `store-exposed`, `store-reset`, `lock-replaced`, `item-duplicate`, `flag-invalid`, `action-malformed`, `conflict-unread`, `unsupported`, `no-performer`, `ambiguous-performer` and the index and query codes; not `tag-refused`, which only a mail tag raises. A refusal calendula words itself SHALL keep the code: a write a source does not support (an online meeting, a scheduled event, a to-do) `unsupported`, an intent no source syncs `no-performer`, several `ambiguous-performer`. The message SHALL be io-pimdir's, but for `store-version` and the refusals calendula words, and SHALL never advise deleting the store.

A store is never migrated (pimdir STORAGE §6): one of another format version than calendula's io-pimdir is refused as `store-version`, whichever way it differs. One naming no format version, or an earlier draft's, SHALL be refused with a message saying the store was written by an earlier sync engine, and giving the reinstall in the order that keeps its queued writes: sync it with the neverest that wrote it until nothing is queued, upgrade neverest, move the store directory aside, then run `neverest init` and sync again; it SHALL never say to delete the store. Any other SHALL be refused with a message saying a newer sync engine wrote it and to upgrade calendula.

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

Each SHALL print `{row, id, calendarId, recurrenceId, partstat, comment, source}`: `row` the queue row the intent was appended as, which `pimdir queue show` and `pimdir queue cancel` take, `id` the event as `event list` prints it, `partstat` `null` for a cancellation, `recurrenceId` and `comment` `null` when not given (pimalaya-cli output.md, *A queue row keeps one shape*).

`source` SHALL name the performer in every payload. In a calendar a source syncing it declares capabilities for, it SHALL be `--source` when given, else the single candidate, else the recorded choice; several candidates and no choice SHALL be refused, listing them. In a calendar whose sources declare nothing, it SHALL be `--source` when given, else the one source syncing the calendar; several sources SHALL be refused, listing them, and none refused, both pointing at `--source`, nothing queued.

`-r/--recurrence-id` SHALL limit either to one occurrence, carried as `recurrence_id`, spelled as the `recurrenceId` an expanded listing shows (the series' `DTSTART` form). When the body is local, a value naming no occurrence of the series, or naming one in another spelling, SHALL be refused. A performer not declaring `calendar.reply.occurrence` or `calendar.cancel.occurrence`, and a store declaring nothing, SHALL be refused with the code `occurrence-unsupported` and an error starting `Occurrence not supported:`, naming the capability and the source, or saying no source of the store declares it; nothing is queued.

A reply to an event naming no `ORGANIZER` or no `ATTENDEE`, or a cancellation of one naming no `ATTENDEE`, SHALL be refused when its body is local, coded `invitation-missing`, the last pointing at `event delete`: an event naming no attendee is the account's own or invites nobody, and the account answers as one of its attendees. An event naming an object whose blob is absent SHALL refuse both coded `body-pending`, as `get_item` refuses its read, rather than queue them unchecked.

#### Scenario: A reply to the account's own event
- GIVEN a pimdir event whose `ORGANIZER` is the account and which names no `ATTENDEE`, as Graph writes one the account made
- WHEN `calendula --json pimdir reply <id> accept` runs
- THEN it exits 1 coded `invitation-missing` and nothing is queued

### Requirement: pimdir reads and retracts its queue
`pimdir queue list`, `pimdir queue show <ROW>` and `pimdir queue cancel <ROW>` SHALL serve the store's queue as Himalaya's do. `queue list` (`ls`) SHALL list the calendar `-k/--calendar` names, refused `calendar-not-found` when the account holds none of that id, or every calendar of the account when it is omitted, as cardamum's lists its address books, and print `{items, parked}`: each creation and each intent waiting, the rows no listing shows (an update or a removal addresses an item a listing shows), then each parked row, any kind, every row in the family's one shape (pimalaya-cli output.md, *A queue row keeps one shape*): `{row, kind, collection, linkId, itemId, producer, queuedAt, attempts, unconfirmed, error, summary}`, every key printed. `kind` SHALL be the action as the store holds it, never re-spelled (`add`, `update`, `calendar-reply`); `linkId` the link id of the item a row creates; `itemId` the stored event a row addresses; `attempts` the sync engine's tries, an unsigned integer; `unconfirmed` whether a parked row's outcome is unknown; `error` `null` while the row is pending, the store's text once it is parked; `summary` the summary of the body a row pins, else of the event it addresses; each `null` where it does not apply.

`queue show` SHALL print `{row, state, collection, kind, attempts, error, appliedAt, seq, refusals}`, `seq` the event an applied row acted on and `refusals` the refusals standing there (pimdir STORAGE §15.4, `PimdirActionStatus::Applied`), each `{collection, source, reason, refusedAt}`, one text line each, `Refused by <source> in <calendar> at <when>: <reason>`, empty but for an applied row; `state` one of `pending`, `parked`, `unconfirmed`, `applied`, `unknown` and `reset`; a row anchored on a calendar of another account SHALL read `unknown`. `queue cancel` SHALL ask before cancelling unless `-y` is given, naming a row its performer sent as one that may have taken effect, and print `{row, kind, unconfirmed}`; it SHALL drop the row as the store's owner, taking that role for the call alone, refused `store-owned` while a sync holds it, and refuse a row applied already or unknown to the account's queue.

`pimdir status` stays the store's report, and `pimdir cancel` the `calendar-cancel` intent: neither is a queue command. `pimdir status` SHALL list the refusals standing in its calendars (`PimdirReader::refusals`, pimdir STORAGE §15.4) under `refused`, each `{calendarId, itemId, source, reason, refusedAt}`, and in the text as columns `EVENT CALENDAR SOURCE REASON` under the parked rows, then `Refused by the server, held until the event is edited or removed`.

#### Scenario: A queued creation cancelled
- GIVEN a pimdir calendar `cal` where `event add` queued the event `Standup`
- WHEN `calendula --json pimdir queue list -k cal` runs, then `pimdir queue cancel -y <row>`
- THEN the first prints its row with `"collection":"cal"`, `"kind":"add"`, `"linkId"` the event's `UID`, `"itemId":null`, `"attempts":0`, `"unconfirmed":false`, `"error":null` and `"summary":"Standup"`, the second `{"row":<row>,"kind":"add","unconfirmed":false}`, and the row leaves the queue

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
