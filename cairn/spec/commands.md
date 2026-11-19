---
cairn: spec
capability: commands
status: current
---

# Commands

The command tree is split into three groups, in this order: the unified API, the protocol-specific escape hatches, and the meta commands. This is the standard Pimalaya CLI split, a portable surface plus per-protocol hatches. The unified API is the union of what the backends serve, held coherent by one meaning per verb, a backend that cannot give a verb's result declaring it unserved rather than giving another.

### Requirement: The unified API is the union of the backends
The `calendar`, `event` and `item` families SHALL expose what any backend serves, each backend declaring what it serves of them. A backend SHALL declare a capability served only when it gives the result every other declarer gives; a backend whose result differs SHALL declare it `none` and keep its own operation in its protocol-specific family, so a verb keeps one meaning. A backend added that cannot serve a unified operation SHALL declare it `none` rather than emulate it.

### Requirement: Capability registry
Each backend SHALL declare, in one table per backend (src/capability.rs) keyed by the name `--backend` takes and compiled into every build, compiled-out backends included, each unified capability: `calendar.list`, `calendar.create`, `calendar.create.id`, `calendar.rename`, `calendar.delete`, `calendar.description`, `calendar.color`, `item.list`, `item.read`, `item.add`, `item.update`, `item.remove`, `item.update.if-match`, `item.remove.if-match`, `online-meeting` and `scheduling.send-updates`, named in pimdir STORAGE Annex B's grammar without the domain word, the collection read as the calendar and the calendar resource as the item. Each SHALL be at a level: `full`; `partial`, with a detail saying what differs; `server`, with a detail saying what the server may refuse; or `none`, with a detail saying why and what serves it instead. The registry SHALL hold the capabilities a unified verb or flag reaches, and nothing a protocol-specific family alone reaches.

### Requirement: An unserved verb is refused before connecting
A unified command SHALL require the capabilities its verb and its flags reach (`--if-match`, `--online-meeting`, `--send-updates`, a calendar's id, name, description or colour) from the backend the account resolves to, before it reads a store, resolves a credential or connects. One the backend declares `none` SHALL fail with the code `capability-missing` and a message naming the backend, the capability and the detail, and, when another backend of the account serves it, that backend and the `--backend` reaching it. A capability served `partial` or `server` SHALL pass, the adapter, the server or the store refusing the case it cannot serve; an adapter MAY keep its own refusal as a second guard.

#### Scenario: A version on vdir
- GIVEN an account whose vdir home holds the event `1` and which configures a CalDAV block too
- WHEN `calendula --json item delete -k cal --if-match x 1` runs
- THEN it fails with `capability-missing` naming `item.remove.if-match`, the vdir backend and `--backend caldav`, the event file untouched and the CalDAV password command never run

### Requirement: A served verb keeps its output
Requiring the capabilities SHALL leave the output of every verb a backend serves unchanged. Only a verb or a flag a backend declares `none` changes: from an uncoded refusal, or from a flag ignored (`--if-match` on a vdir update), to `capability-missing` before anything is reached.

### Requirement: Protocol-specific families
`caldav`, `msgraph`, `gcal`, `pimdir` and `vdir` SHALL each expose what only that backend has, gated behind its own cargo feature, shaped as cardamum's: CalDAV a flat list of WebDAV method names ("caldav speaks the protocol"), Microsoft Graph and Google Calendar nested by the API's resource, each resource named by the API's own spelling, plural included, with its singular as a hidden alias, vdir flat collection verbs plus an `item` subtree. `caldav mkcalendar` and `vdir create` SHALL take the calendar's metadata with the letters `calendar create` and `calendar update` use: `-n`, `-d/--description`, `-C/--color`, the former `-D` read as `-d`.

gcal covers the half of the Calendar API iCalendar cannot express: sharing, availability, recurrence expansion, server-side parsing and the palettes. vdir covers its collection verbs including `rename`, which the unified API has no home for, and its raw items.

pimdir covers `status`, reporting the account being read, every account the store groups collections under, how much of each calendar is downloaded, how many creations are queued for the next sync, and every action the store's owner parked in those calendars, by its queue row handle (ROW), its calendar, its kind and the error that parked it, `{row, handle, calendarId, kind, error}`, `kind` kebab-case as the queue spells it; `queue list`, `queue show` and `queue cancel`, the store's queue as Himalaya reads and retracts it; `reply` and `cancel`, the scheduling intents only its sync engine can perform; and `search` and `count`, the store's search index queried in the pimdir SEARCH language.

A backend MAY exist without a family where its protocol adds nothing the unified surface lacks; a family SHALL NOT push protocol-specific concepts into the unified API instead.

### Requirement: gcal family
`gcal` SHALL cover: `calendars`, the richer listing carrying the access role, the primary flag, the time zone and the default reminders a shared `Calendar` has no room for; `acl` with `list`, `create`, `update` and `delete` over the sharing rules; `free-busy`, the availability query over a window.

It SHALL also cover `events`, the Calendar API's Events resource (hidden alias `event`): `events instances`, the occurrences a recurring event expands to; `events move`, relocating an event to another calendar; `events quick-add`, an event parsed server-side from a sentence. And `colors`, the two palettes the API's colour ids refer to, and `settings`, the user's own. `gcal instances`, `move` and `quick-add` SHALL stay hidden commands running the same ("A former command name stays hidden").

An access role, a scope type or an event status the API returns and calendula does not know SHALL print as the API spells it, in text and in JSON, never fail the listing. The `acl` flags SHALL keep taking the values calendula knows alone.

An ACL rule id is `<scope type>:<scope value>` (`default` for everyone), which the API mints, so `create` SHALL take the scope and the role as flags and derive it, `update` SHALL read the scope from the id it is given, its `--scope` and `--value` optional and refused `usage-invalid` before any request when they disagree with the id (the value compared case-insensitively), as an id naming no scope is, and `delete` SHALL accept the id a listing showed. `create` SHALL refuse, before any request, a `user`, `group` or `domain` scope with no `--value`, and a `default` scope given one, rather than drop it and grant everyone.

`free-busy` and `events instances` SHALL take `--from` / `--to` as inclusive days, as `event list` does, so one date spelling serves the whole CLI. `free-busy` SHALL need both, the API answering no window without them.

`calendars`, `acl list`, `events instances` and `settings` SHALL read every page and print under `--json` every item as Google sent it, read as JSON and never through a type (pimalaya-cli output.md, *Protocol commands are raw-faithful*), under `calendars`, `rules`, `instances` and `settings`; their text reads the same items. `request <METHOD> <PATH> [JSON]` SHALL send any Calendar API request, the path joined onto the `v3/` base or a full URL, and print `{response}`, the JSON Google answered, as `msgraph request` and cardamum's `gpeople request` do.

`free-busy` and `colors`, each a single read, SHALL print under `--json` what the API answered, as Google sent it, every property kept, as the `msgraph` reads of one calendar, one event and the profile print Graph's object (the family's ruling of 2026-11-18 that a single protocol read prints the provider's payload raw), their text forms unchanged.

### Requirement: What the gcal family declines
`channels` / `watch` SHALL NOT be exposed: a push channel delivers to an HTTPS endpoint the caller must host, which a CLI has not. `calendars.transferOwnership` SHALL NOT be exposed: it is an irreversible administrative act on a Workspace domain.

`calendars.clear` SHALL NOT be exposed: it empties the primary calendar with no undo, and `calendar delete` covers the secondary case. These are declined on purpose, and recorded as such rather than left as gaps.

### Requirement: A former command name stays hidden
A protocol command renamed or moved SHALL keep its former name, hidden from the help, the manual and the completions, so a script keeps working: an alias where the command keeps its depth (`caldav list` and `ls` for `caldav propfind`, `caldav create` for `caldav mkcalendar`), a hidden command running the same code where it moved deeper (`gcal instances` and `instance`, `gcal move` and `gcal quick-add` for `gcal events instances`, `move` and `quick-add`), each printing what its new name prints. A hidden command keeps its JSON schema key (`calendula-gcal-instances`); an alias has none of its own, its schema being its command's (`calendula-caldav-propfind`).

#### Scenario: A former name
- GIVEN a CalDAV account and a gcal one
- WHEN `calendula caldav list`, `calendula caldav create work` or `calendula gcal quick-add "Lunch tomorrow"` runs
- THEN it runs `caldav propfind`, `caldav mkcalendar work` or `gcal events quick-add "Lunch tomorrow"`, printing what that prints, and none of the three shows in `--help`

### Requirement: A protocol refusal is a result
Inside the `caldav`, `gcal` and `msgraph` namespaces, an HTTP 4xx or 5xx the server answers (the `request` commands included) SHALL print as the command's result and exit 3, as himalaya's `gmail`, `msgraph` and `jmap` do (pimalaya-cli cairn/spec/output.md, *A protocol refusal is a result*): `{refusal: {status, reason, message, code, retryAfter}}`, `status` the HTTP status; `reason` the provider's name for it in its spelling (Google's first reason, Graph's `error.code`), `null` when it names none, as a WebDAV server does; `message` the provider's text, a WebDAV server's body; `code` the code the failure would carry, `null` when none applies; `retryAfter` the seconds the server asked to wait, `null` when it asked none. A command getting no answer (a network, TLS or decoding failure, a local error, a value refused before sending) SHALL print the failure object and exit 1. Each command's JSON Schema SHALL describe its own result and the refusal as two alternatives told apart by their keys. The unified commands are not concerned: a refusal there fails as before.

#### Scenario: A Graph request refused
- GIVEN a Graph account answering 403 `ErrorAccessDenied`
- WHEN `calendula --json msgraph request get me/calendars/nope` runs
- THEN it prints `{"refusal":{"status":403,"reason":"ErrorAccessDenied","message":"…","code":"permission-denied","retryAfter":null}}` and exits 3

### Requirement: caldav speaks the protocol
`caldav` SHALL be the WebDAV and CalDAV methods (RFC 4918, 4791, 6578) run against the account's endpoint, each named after its method, as cardamum's `carddav` is: `discover`, `propfind`, `proppatch`, `mkcalendar`, `report` (`query`, `multiget`, `sync`, `raw`), `get`, `put` and `delete`. Each takes its calendar and item ids positionally, and refuses one no CalDAV resource can have before anything connects (backends, "A CalDAV id addresses its own resource").

- `discover` SHALL print `{server, principal, calendarHomeSet}`, an endpoint it cannot resolve `null` rather than a failure, as `Fields`.
- `propfind` with no calendar SHALL list the calendars under the home set, `{"calendars": [{id, displayName, description, color, components, ctag, syncToken}]}`; with one, the items it holds by id and entity tag, no body (a `calendar-query` REPORT asking `getetag` alone, as io-webdav enumerates a collection, and as cardamum's `carddav propfind` does), `{"items": [{id, etag}], "truncated"}`, `truncated` when a 507 row cut the listing.
- `proppatch <ID>` SHALL set the display name (`-n/--name`), the description (`-d/--description`), the colour (`-C/--color`) and the time zone (`--tz`, a whole VTIMEZONE) it is given, in one PROPPATCH, refusing a line giving none; `mkcalendar <ID>` SHALL make a calendar with the same flags plus `--component` (repeatable, RFC 4791 §5.2.3), `--display-name` and `-D` read as `-n` and `-d`. Both print `{id}`.
- `report query <CALENDAR>` SHALL send a `calendar-query` (RFC 4791 §7.8) whose filter is built from its flags: `--component KIND` the component filter under VCALENDAR, `--from` and `--to` its `time-range` as inclusive days, `--match PROP VALUE` and `--not-match PROP VALUE` a prop-filter whose `text-match` is a substring match (negated for the second), `--defined PROP` and `--not-defined PROP` a prop-filter present or `is-not-defined`, `--collation` the collation of every match. A window or a prop-filter given without `--component` SHALL filter `VEVENT`; no flag at all asks for every item. `report multiget <CALENDAR> <ITEM-ID>...` SHALL send a `calendar-multiget` (§7.9). Both SHALL print `{"items": [{id, etag, contents, encoding}], "truncated", "errors"}`, the bytes as "An item's bytes are text or base64" says, and an `errors` entry per row the server answered without the item's data, coded as a CalDAV listing row is ("Each backend names the items it could not read"), a multiget row answered 404 or 410 `item-not-found`; a non-empty `errors` exits 3. The text SHALL print `ID`, `ETAG` and `SUMMARY` columns, the summary the item's first one, then the errors last.
- `report sync <CALENDAR>` SHALL send a `sync-collection` (RFC 6578) from `--sync-token`, or a Depth 1 PROPFIND under `--fallback`, and print `{changed: [{href, etag}], vanished, syncToken, truncated}`. `report raw <CALENDAR> <XML>` SHALL send the XML given (a file, inline, or `-` for stdin) to the path io-webdav builds for the calendar (`rfc4918::collection_path`) and print `{responses: [{href, status, etag, dataBytes}], syncToken}`, `dataBytes` the size of the row's whole `calendar-data`, its final CRLF included.
- `get <CALENDAR> <ITEM-ID>` SHALL print `{id, etag, contents, encoding}`, its text the iCalendar; `put <CALENDAR> <ITEM-ID> <ICAL>` SHALL write the iCalendar given (a file, inline, or `-`) as it is, unconditionally, gated on `--if-match`, or create-only under `--if-none-match`, and print `{id, etag}`; `delete <CALENDAR> [ITEM-ID]` SHALL delete that item, gated on `--if-match`, or the whole calendar without one, refusing `--if-match` there, and print `{id}`. A request naming one item the server does not hold SHALL fail `item-not-found`, the server's answer kept.

#### Scenario: A calendar listed by PROPFIND
- GIVEN a CalDAV home holding the calendar `cal`
- WHEN `calendula --json caldav propfind`, then `calendula --json caldav list`, runs
- THEN both print `{"calendars":[{"id":"cal",…,"components":[…],"ctag":…,"syncToken":…}]}`

### Requirement: msgraph family
`msgraph`, gated on the `msgraph` feature, SHALL be nested by Graph resource, each command named after its Graph operation and printing Graph's raw objects under `--json`, as cardamum's `msgraph` is: `calendars` (`list`, `get`, `create`, `rename`, `delete`), `events` (`list`, `get`, `create`, `update`, `delete`, `delta`), `profile get` (`/me`) and `request <METHOD> <PATH> [JSON]`, the escape hatch printing `{"response"}`. `calendars list` SHALL print `{"calendars": [...], "nextPage"}` and `events list` `{"events": [...], "nextPage"}`, one Graph page, `nextPage` Graph's `@odata.nextLink` lifted out of the page, `null` on the last one, which `--page-token` takes back as is, the link carrying the calendar and the query, so `-k`, `--top`, `--select` and `--filter` are refused beside it; `events delta` one page of the calendar view delta over `--from` and `--to` (inclusive days, required without `--delta-link` or `--page-token`), `{"events": [...], "nextPage"}`, `nextPage` the `@odata.nextLink` draining the round, taken back by `--page-token`, the round's last page keeping Graph's raw `@odata.deltaLink`, which `--delta-link` takes back for the next round (pimalaya-cli output.md, *Paging resumes*). Their text SHALL end with a `Next page: <cursor>` line when one follows. `events create` and `update` SHALL take a raw Graph event JSON body (a file, inline, or `-` for stdin), `update` a PATCH; `-k/--calendar` names the calendar of `events list`, `create` and `delta`, the default one when omitted. A delete prints `{id}`. A request naming one event Graph does not hold, a 404 or a 400 `ErrorInvalidIdMalformed`, SHALL fail `item-not-found`; one naming a calendar so, `calendar-not-found`, Graph's message kept. The text prints a list as records, one item as fields. `calendars list` and `events list` SHALL take `--top` long-only, as Himalaya's Graph listings do, the former `-s` read as `--top` and named by no help, manual or completion.

#### Scenario: The Graph listings take `--top` long-only
- WHEN `calendula msgraph events list --help` prints
- THEN it lists `--top <N>` with no short letter
- AND `calendula msgraph events list -s 5` reads as `--top 5`

### Requirement: vdir list carries the collection's metadata
`vdir list` SHALL print each collection `{id, displayName, description, color, path}` under `--json`, each of the three `null` when its marker file is absent, as cardamum's does; its text SHALL be the columns ID, PATH, COLOR, NAME, the description under `--json` alone, as `calendar list` prints it.

### Requirement: vdir items
`vdir item` (visible alias `items`) SHALL cover the raw files of a collection, byte for byte and of any kind, as cardamum's `vdir item` does: `list <COLLECTION>` (`{"items": [{id, kind, size, path}]}`), `get <COLLECTION> <ID>` (`{id, kind, contents, encoding}`, its text the bytes), `add <COLLECTION> [--kind ical|vcard] <INPUT>` (visible alias `new`, `create` a hidden one; the kind sniffed from the bytes when omitted, `{id}`), `update <COLLECTION> <ID> <INPUT>` (its kind kept), `delete`, `copy <SOURCE> <TARGET> <ID>` and `move <SOURCE> <TARGET> <ID>`, each write printing `{id}`. A collection that is not one directory name of the home SHALL fail `calendar-not-found`, an item it does not hold `item-not-found`. `copy` and `move` into a collection holding an item of that id SHALL fail `item-duplicate`, ``Cannot <verb> item `<id>` into `<target>`: it holds an item of that id already``, both items left as they were, as cardamum's do.

#### Scenario: A raw item is added
- WHEN `calendula vdir item add cal --kind ical -` runs
- THEN it stores the item read from stdin in `cal` and prints `{id}`
- AND `vdir item create` and `vdir item new` run the same command, and `calendula vdir item --help` lists `add` (alias `new`) and not `create`

### Requirement: The component families against item
The unified API SHALL offer two kinds of view over the same resources. `item` is the raw, unfiltered one: it lists, reads, writes and deletes any iCalendar object by id, leaving the bytes untouched, a to-do or a journal entry included.

`event` is the projected one: it keeps VEVENT, renders the columns an event is read by, and draws a cal(1)-style agenda.

Both SHALL share the `-k/--calendar` selector and the same item API; only the rendering and the filter differ.

### Requirement: One family per stored component kind
calendula SHALL be the events CLI: the unified API SHALL offer one projected family, `event`, for VEVENT. VTODO belongs to unfog and VJOURNAL to journal, one CLI per iCalendar domain, since CalDAV holds one component type per calendar object resource (RFC 4791 4.1) and the providers run events, tasks and notes as separate services.

`event` SHALL cover `list`, `read`, `search`, `add`, `update` and `delete` over the item API, and `item` `list`, `read`, `add`, `update` and `delete`: the verbs of Himalaya, `add` storing an item as `message add` does and `search` looking events up as `envelope search` does, `create` and `find` kept as their hidden aliases. An item whose bytes do not parse SHALL project nothing rather than fail the listing, the listing naming it in its `errors` ("A listing names the items it could not read").

#### Scenario: A to-do in an event calendar
- GIVEN a CalDAV calendar holding a VEVENT and a VTODO
- WHEN `calendula event list` and `calendula item list` run on it
- THEN the event listing shows the VEVENT alone, and the item listing shows both

### Requirement: Columns follow the kind
An event listing SHALL show each event's summary and its start and end.

The start and end SHALL show for a person to read, the JSON `start` and `end` in the local zone (`TZ`, then the system's), the one a floating time is read in: `2026-10-30 09:00` for a timed event, the days for a whole-day one with the end as its last day (`2026-05-19`), never before its first, so a zero-length one (`DTEND` equal to `DTSTART`) ends on its start. A time that did not resolve SHALL show as iCalendar spells it. Nothing is truncated. The `--json` payload SHALL stay as "The event projection" states it, a whole day's `end` exclusive.

A property the event omits SHALL print as `-` rather than nothing, so the columns line up down the list.

### Requirement: Only events draw an agenda
`agenda` SHALL stay a VEVENT command: the events listed under its grid, by the day and time they start, are VEVENTs, an event being what occupies time.

### Requirement: An agenda shows every event at an instant
The agenda SHALL render every event whose start falls at a given instant, never one of them, and SHALL order two events at one instant totally and stably, by label then by item id, so the same calendar renders the same way twice.

The events at one instant SHALL print under one time column rather than repeating it, so a duplicated pair reads as two events rather than as two rows that happen to share a time.

Two events may legitimately start at the same moment: two unrelated meetings at 09:00, and, since a collection may hold two resources under one `UID` (pimdir STORAGE §9), two copies of what a server considers one identity.

A view keyed by the instant alone shows the last one written and reports nothing, which loses exactly the event the store went to the trouble of keeping.

The `--json` payload SHALL carry the same multiplicity, so a consumer reading it is not told there was one event where there were two: `{events, errors}`, `events` listing every event under the grid as `{id, start, label}`, in the same order the text renders them, `id` the item id `event read` takes, `start` the event's as "The event projection" prints it, `label` what the text prints, and `errors` it lists the items the agenda could not read ("A listing names the items it could not read").

No second shape SHALL be kept beside it, since one that still dropped an event would leave the defect reachable.

### Requirement: An agenda expands recurring series
The agenda SHALL read the events over the days its grid draws and SHALL expand a recurring series over them, as a windowed `event list` does: each occurrence lists under the grid at its own day and time, overrides applied and excluded dates dropped.

A series placed once at its first start leaves a yearly event out of every year but its first, so the agenda and a listing of the same days would disagree.

The agenda SHALL stamp each event, and file it under its day, at its start in the local zone `event list` reads (`TZ`, then the system's), a timed event at its instant there, a whole-day one on its day, a floating one at its wall time. Its JSON `start` SHALL stay the event's instant as "The event projection" prints it, whatever the local zone.

#### Scenario: A yearly event created years ago
- GIVEN a calendar holding a yearly event whose first start is in 2019
- WHEN `calendula event agenda 2027` draws that year
- THEN the event lists under the grid at the day it falls on in 2027

#### Scenario: A UTC event under a local zone
- GIVEN `TZ=Europe/Paris` and an event `DTSTART:20261112T090000Z`
- WHEN `calendula --json event agenda 11 2026` runs
- THEN the grid lists it at `Nov 12, 10:00`, as `event list` prints it at `2026-11-12 10:00`, and its JSON `start` is `2026-11-12T09:00:00Z`

### Requirement: An agenda names months and weekdays in the time locale
The agenda SHALL draw month and weekday names, and stamp the events under its grid, in the locale `LC_ALL`, then `LC_TIME`, then `LANG` resolves to, as cal does. A locale that is unset, `C`, `POSIX` or unknown SHALL fall back to the English names of the POSIX locale. A month argument SHALL be read in English or in the language of that locale.

The grid is a port of cal, so the two drawing the same month in different languages on one system would disagree.

#### Scenario: A Spanish system
- GIVEN `LANG=es_ES.UTF-8` and no `LC_ALL` or `LC_TIME`
- WHEN `calendula event agenda -m` draws October 2026
- THEN the header reads `octubre 2026`, the weekdays `lu ma mi ju vi sá do`, and an event on the 19th is stamped `oct 19`

#### Scenario: The POSIX locale
- GIVEN `LANG=C`
- WHEN `calendula event agenda -m` draws October 2026
- THEN the header reads `October 2026` and the weekdays `Mo Tu We Th Fr Sa Su`

#### Scenario: A double-width language
- GIVEN `LANG=ja_JP.UTF-8`
- WHEN `calendula event agenda -m` draws October 2027
- THEN the header reads `10月 2027` and the weekdays `日 月 火 水 木 金 土`, both lines 20 columns wide like the days under them

### Requirement: An agenda reads its date as cal does
The agenda SHALL read one argument that is a number as a year and drawn whole, and one argument that is not a number as a month name, drawing that month of the current year. A day that is not a number of the month, a month that matches no name in English or in the time locale, a month number outside 1-12, or a year that is not an integer SHALL be refused, naming the value, never replaced by the 1st, by January or by the current year.

The grid is a port of cal, and a typo silently drawing another month reads as an empty one.

#### Scenario: A lone month name
- GIVEN `LANG=es_ES.UTF-8` in 2026
- WHEN `calendula event agenda noviembre` runs
- THEN it draws November 2026 alone, as `cal noviembre` does

#### Scenario: An unknown month name
- GIVEN `LANG=es_ES.UTF-8`
- WHEN `calendula event agenda novembre 2026` runs
- THEN it fails with `Unknown month name: novembre` and draws nothing

### Requirement: A page of a family holds that family
A listing SHALL narrow by component kind before it paginates. Filtering after paginating makes a page mean "the components of this kind among the first N items of any kind", which on a calendar dominated by another kind is empty however many matches the calendar holds.

The kind SHALL travel with the page and the window as one query, so no backend can serve one without the others. A backend SHALL push the kind down where its protocol defines such a filter: CalDAV as an RFC 4791 `comp-filter`, with the `time-range` nested inside it as 9.7.1 requires. A backend modelling one kind only SHALL answer any other kind without a round-trip.

The raw `item` family names no kind and SHALL keep every one, which is what makes it the unfiltered view.

A pimdir item whose body is not local SHALL answer its kind from the summary table the store filed it in (event, task or journal), which names the component a reader renders the resource as, so narrowing by kind keeps the listing availability-aware.

### Requirement: A component window is applied locally
`event list` pushes its window down where the backend can narrow server-side, and applies it after parsing elsewhere. `item list` takes no window.

A component carrying no date at all SHALL show only in an unfiltered listing.

### Requirement: Kinds with no family
VFREEBUSY and VTIMEZONE SHALL NOT get a family.

A VFREEBUSY is the answer to a query rather than a resource a calendar stores, and where a backend offers such a query it belongs to that backend's own protocol-specific family. A VTIMEZONE defines the zones the other components reference and is not an item a user lists.

### Requirement: Projections never rewrite bytes
Projecting a VEVENT out of an item SHALL be read-only and lossy by design: calendula returns what the backend holds. An item whose bytes do not parse SHALL project no event rather than fail the listing, so one malformed resource cannot hide a whole calendar, and the listing SHALL name it in its `errors`.

"What the backend holds" is verbatim for the backends that store iCalendar, and synthesized for the one that does not; the [projection](./projection.md) capability governs the difference, and no command layer SHALL rewrite bytes of its own.

### Requirement: Time-range filtering
`event list` SHALL accept `--from` and `--to` as inclusive local days (`TZ`, then the system's zone), as the rows, the agenda and `pimdir search date:` read a day. The pair SHALL map onto a range from the UTC instant of `--from`'s local midnight to the one of the local midnight after `--to`, exclusive, so `--to` covers the whole day named; the agenda SHALL fetch the days its grid draws the same way. Inside the range, a UTC time SHALL be tested as an instant, and a date or a local time by the local days the bounds fall on. Every other command taking `--from` and `--to` (`caldav report query`, `gcal free-busy`, `gcal events instances`) SHALL read them alike.

#### Scenario: An evening west of UTC stays on its day
- GIVEN `TZ=America/New_York` and an event at `DTSTART;TZID=America/New_York:20261130T200000`
- WHEN `calendula event list --from 2026-11-30 --to 2026-11-30` runs, or `calendula event agenda 11 2026`
- THEN the event is listed

A crossed pair SHALL be rejected by name. A range SHALL lift the default page-size cap, so a window returns every match rather than its first page. A window given `-s` SHALL page its occurrences, sorted by start across the whole window, the page line counting the occurrences it shows, never the items they came from. A recurring series SHALL be walked from the window's start (its length and a day of offsets either way before it), however long before the series began, so no step cap cuts it.

A backend filtering locally (vdir, msgraph, pimdir) SHALL keep an item when one of its events starts in the window or one of its occurrences overlaps it. An undownloaded pimdir series the summary says recurs SHALL pass from its `DTSTART` to its `UNTIL`.

### Requirement: A window expands recurring events
With a window, `event list` SHALL list every occurrence overlapping it rather than every VEVENT: the recurrence set of each `UID` (`DTSTART`, `RRULE`, `RDATE`, minus `EXDATE`, with its `RECURRENCE-ID` overrides) walked by ical-rs, an override standing in for the instance it names, and one moved into the window from an identity past it still listed. An override travelling without its series lists on its own.

A `UNTIL`, an `EXDATE`, an `RDATE` or a `RECURRENCE-ID` written on another clock than the series' `DTSTART` (UTC beside a zoned start, as RFC 5545 3.3.10 requires of `UNTIL`) SHALL be told on the `DTSTART`'s clock before it bounds, removes, adds or overrides an instance, through the zone each `TZID` resolves to.

An occurrence SHALL carry the item `id` and its `recurrenceId`, the instance identity spelled as the series' `DTSTART` (a date, a local time, or a `Z` time), which `event read -r` takes back. Occurrences SHALL list in start order. A timed occurrence overlaps when it starts before the window's end and ends after its start (a zero-length one when it starts inside); a whole-day one compares by date. A window open on its end SHALL reach a year past its start.

Without a window the listing SHALL be the raw one, in the order the backend returns the items, a series once at its first start; it SHALL NOT be sorted by start, since only the first page the backend returns would be, which reads like an agenda without being one. The help SHALL point to `--from` (or `event agenda`) for what is coming.

### Requirement: Times resolve to instants
A time SHALL resolve through its `TZID` as the time-zone database names it (a writer's path prefix or a leading slash tolerated), else through the calendar's `VTIMEZONE` of that `TZID` when it states at least one observance, else in the local zone, as a floating time does; a `Z` time is UTC. A local time a transition skips SHALL read with the offset before the gap, one it repeats as its first occurrence (RFC 5545 3.3.5).

Expansion SHALL move a UTC `UNTIL` onto the wall clock of a zoned `DTSTART` before the civil walk, and SHALL drop the instances a calendar-defined zone skips.

### Requirement: The event projection
An event's JSON SHALL carry `id`, `etag` (the entity tag of its item, `null` when the backend has none), `uid`, `recurrenceId`, `summary`, `description` and `location` (`null` when the event carries none, never `""`), `start` and `end` (RFC 3339 with the offset, a zero one spelled `Z` as pimalaya-cli's output contract and Himalaya spell UTC, or `YYYY-MM-DD` with an exclusive end for a whole day, `null` when the time does not resolve), `allDay`, `timeZone`, `zoneAssumed` (`true` when `DTSTART` or `DTEND` names a `TZID` neither the time-zone database nor the calendar defines, read in the local zone), `recurring`, `sequence` (its `SEQUENCE`, 0 when absent), `status`, `transparency`, `busyStatus` (Outlook's), `organizer` (`email`, `name`), `attendees` (`email`, `name`, `partstat` defaulting to `NEEDS-ACTION`, `role`, `rsvp`, `cutype`), `onlineMeetingUrl` (`CONFERENCE`, then the vendors' properties) and `alarms` (every VALARM of the VEVENT in document order: `action` uppercased and `trigger`, its value as written, each `null` when absent). The end SHALL be `DTEND`, else the start plus `DURATION`, else a day for a date and the start for a time. A time SHALL print in that one form: its iCalendar spelling stays in the item's contents.

`event read` SHALL print the item's bytes, VALARMs included, and its JSON SHALL be the row of its item ("An item's bytes are text or base64") plus the projected `events`, or the one occurrence `-r/--recurrence-id` names.

### Requirement: An item's bytes are text or base64
An item SHALL print as one row in `item list`, `item read` and `event read`: `{id, calendarId, etag, contents, encoding}`, `etag` `null` when the backend versions no item. `contents` SHALL be the iCalendar text when its bytes are valid UTF-8, `encoding` then `utf-8`, and the base64 of the bytes otherwise (RFC 4648 standard alphabet, padded), `encoding` then `base64`, as cardamum prints a vCard. A read SHALL always carry both, an empty file the empty text; `item list` alone SHALL print both `null`, for an item carrying no bytes (a pimdir item whose body is not local yet or cannot be read), which a read refuses. No output SHALL decode an item's bytes lossily under `--json`.

#### Scenario: A body that is not UTF-8
- GIVEN the vdir calendar `personal` holding `a.ics`, whose `SUMMARY` holds the Latin-1 byte 0xE9
- WHEN `calendula --json item list`, `item read a` or `event read a` runs
- THEN each prints the file's bytes in base64 as `contents` and `"encoding":"base64"`, and `calendula item read a` prints `SUMMARY:Caf` and U+FFFD

### Requirement: An event is found by its UID
`event search <UID>` SHALL list, unexpanded, every VEVENT of the calendar carrying that `UID`, with the id the other `event` commands take. `find` SHALL stay its hidden alias.

### Requirement: A UID hint narrows a search
The listing query MAY carry a `UID`, which `event search` sets. A backend MAY leave out an item it can tell carries another `UID` without reading it, and SHALL ignore the hint on a listing naming a page size, which leaving items out would shift. pimdir rules an item out by its summary, and Google Calendar SHALL send it as the `iCalUID` filter of `events.list`, so a search reads the series and its exceptions alone rather than every page of the calendar. The caller SHALL still compare the `UID` of every event it gets.

### Requirement: An event is named by its id alone
`event read`, `event update` and `event delete` SHALL take the id `event list` reports and never an iCalendar `UID`, which `event search` turns into that id: resolving a `UID` costs a listing of the whole calendar on every backend but pimdir. `event delete` SHALL read the id first: a read answering `item-not-found` SHALL fail the command coded `item-not-found`, its message naming `event search <UID>` ("A missing item reads one sentence"), and list nothing; any other failure of the read SHALL fail the command with its code; a read answering `body-pending` (pimdir) names the item all the same. The item SHALL be deleted whole.

#### Scenario: A UID given to event delete
- GIVEN the vdir calendar `personal` holding `a.ics`, whose `UID` is `a@example.org`
- WHEN `calendula --json event delete a@example.org` runs
- THEN it fails coded `item-not-found`, naming `event search <UID>`, and `a.ics` stays

### Requirement: A missing item reads one sentence
A unified command naming an item its calendar does not hold SHALL fail coded `item-not-found` with one sentence whatever the backend, Himalaya's for a missing message: `item read`, `item update` and `item delete` ``Item `<id>` not found in calendar `<calendar>` ``, and `event read`, `event update` and `event delete` ``Event `<id>` not found in calendar `<calendar>`; an event is named by the id `event list` shows, `event search <UID>` finding it by its iCalendar UID``. The backend's own words SHALL follow as its `sources`, its one source: a read SHALL NOT list a ``Read item ...`` line beside them. A vdir `item update` or `event update` given an iCalendar for an id the calendar does not hold SHALL fail so, writing nothing, as every other backend does.

#### Scenario: An unknown id on a vdir calendar
- GIVEN the vdir calendar `personal` holding no item `nope`
- WHEN `calendula --json item read nope` and `calendula --json event delete nope` run
- THEN the first fails coded `item-not-found` with ``Item `nope` not found in calendar `personal` ``, the second with ``Event `nope` not found in calendar `personal`; ...``, each naming io-vdir's miss in `sources` alone
- AND `calendula --json item update nope <ICAL>` fails the same and writes no `nope.ics`

### Requirement: A delete can be gated on a version
`event delete` and `item delete` SHALL take `--if-match <ETAG>`. CalDAV and gcal SHALL send it as `If-Match`, msgraph SHALL compare it with the event's `changeKey`, and pimdir SHALL check it as an update does. A backend that cannot gate a write (vdir) SHALL declare `item.update.if-match` and `item.remove.if-match` `none`, so `--if-match` on an update or a delete is refused with `capability-missing` rather than dropped.

`item read` SHALL print the row of its item ("An item's bytes are text or base64"), its `etag` the version read, `null` when the backend has none.

### Requirement: A JSON error carries a stable code
A failure a caller is expected to act on SHALL carry a stable code, printed under `--json` as a `code` string beside `error`, `sources` and `backtrace`, and in an `errors` entry beside `id`, `error` and `sources`. The code SHALL be found anywhere in the error chain, so added context does not hide it, or read from the typed error of the protocol that refused ("A server's refusal carries its code") or from pimalaya-config's `AccountError` (config, "Account resolution failures name what is missing"). A failure with no code SHALL print `"code": null`. The plain output SHALL stay unchanged. Codes are kebab-case and never renamed; `error` stays free wording. Every code SHALL be declared in every build, whatever the backends compiled in.

| Code | Raised when |
|---|---|
| `account-missing` | no account has the name `-a` gives, `default` included; nothing was read |
| `account-default-missing` | no `-a` names an account and none is marked `default = true`; nothing was read |
| `account-default-several` | no `-a` names an account and several are marked `default = true`, the configuration layers merged; nothing was read |
| `backend-not-built` | the account's backend is one this build leaves out, its block loaded unread: the message names the cargo feature bringing it back; nothing was read or connected |
| `capability-missing` | the backend the account resolves to does not serve a capability the command reaches; nothing was read, opened or connected |
| `stdin-taken` | a command run inside a session (`repl`, `mcp`) would read its iCalendar from stdin, which carries the session's commands; nothing was read |
| `item-not-found` | the item a command names is not in the calendar: the failure of `item read`, `item update`, `item delete`, `event read`, `event update` or `event delete`; an item id no CalDAV item can have, before anything connects (backends, "A CalDAV id addresses its own resource") |
| `calendar-not-found` | the calendar a command names does not exist, or no calendar can have its id (not one vdir directory name, refused by io-webdav on CalDAV); nothing was read or written |
| `calendar-duplicate` | a create names a calendar that already exists (backends, "A calendar created twice is a duplicate"); nothing was written |
| `calendar-default-missing` | a unified or pimdir command names no calendar and the account sets no `calendar.default`; nothing was read |
| `item-unreadable` | the backend answered for an item calendula could not read: its file could not be read (vdir), its iCalendar did not parse, the server refused it while it was read again (Graph), its body could not be read (pimdir); an `errors` entry |
| `item-invalid` | the server refused the item written as no iCalendar it takes (CalDAV, Google Calendar, see backends "A server's refusal carries its code"); nothing was written, and the item is what to fix, not the account's rights |
| `permission-denied` | the server refused the request to the account (see "A server's refusal carries its code") |
| `quota-exceeded` | the write would exceed the account's quota |
| `rate-limited` | the server throttled the request; calendula retries nothing |
| `body-pending` | the failure of `item read` or `event read` on a listed item whose body is not local (pimdir): never fetched, or its blob file gone or failing its name; a sync brings it |
| `precondition-failed` | a write's `--if-match` names a version the item no longer has, on any backend that takes one |
| `occurrence-unsupported` | `pimdir reply` or `pimdir cancel` names an occurrence its performer cannot act on alone |
| `invitation-missing` | `pimdir reply` names an event naming no `ORGANIZER` to answer or no `ATTENDEE` to answer as, or `pimdir cancel` one naming no `ATTENDEE` to notify; nothing was queued |
| `item-duplicate` | a create names a `UID` its calendar holds: a CalDAV `no-uid-conflict` or 412, a gcal calendar holding it, a pimdir calendar holding or queuing it |
| `store-uncreated` | the pimdir store has no schema yet: its sync engine creates it |
| `store-version` | the pimdir store is of another format version than calendula's: an earlier sync engine's is drained by it then set aside, a later one's needs calendula upgraded |
| `store-foreign` | the pimdir database is not a pimdir store: another program wrote it |
| `store-corrupt` | the pimdir store's two version stamps disagree, or its schema is not the canonical one |
| `store-not-current` | the pimdir store holds no schema yet and the verb creates none: its sync engine creates it |
| `store-hash-algo` | the pimdir store names its bodies by a hash this calendula does not know: upgrade calendula |
| `sync-version` | the pimdir store's sync state was written by another version of its sync engine |
| `store-account-ambiguous` | `pimdir.account` is unset and several accounts of the pimdir store hold calendars; name one |
| `store-account-unknown` | `pimdir.account` names no account of the pimdir store |
| `store-busy` | another process holds the pimdir store; retry |
| `store-owned` | a sync owns the pimdir store; retry once it ends |
| `store-staging` | a producer is staging a body in the pimdir store; retry |
| `store-in-flight` | a pimdir store verb is between two of its chunks; retry |
| `store-exposed` | a path of the pimdir store is a link or another user can reach it: restrict it |
| `store-reset` | the pimdir store is not the one that answered a queue row's handle, a copy restored since; nothing was done with the row |
| `lock-replaced` | the lock file of the pimdir store was removed or replaced under a write: nothing was queued; retry |
| `flag-invalid` | a pimdir write carries a flag the store cannot hold |
| `action-malformed` | a pimdir queue action's payload is one its schema refuses |
| `conflict-unread` | the pimdir conflict holds a body nobody has read: fetch it first |
| `unsupported` | a source of the pimdir store does not support the write (an online meeting, a scheduled event, a to-do among them); nothing was queued |
| `no-performer` | no source of the pimdir store can perform the intent (`pimdir reply`, `pimdir cancel`), or a write (`event add`, `update`, `delete`, `item add`, `update`, `delete`) to a calendar no source syncs; nothing was queued |
| `ambiguous-performer` | several sources of the pimdir store can perform the intent and none is chosen; nothing was queued |
| `index-missing` | the pimdir store has no search index yet: its sync engine builds it |
| `index-outdated` | the pimdir search index was built otherwise or from another store: its sync engine rebuilds it |
| `index-version` | the pimdir search index is of another draft or shape: its sync engine rebuilds an earlier draft's, and leaves any other |
| `index-zone-unknown` | the pimdir search index reads floating times, or the search reads days, in a zone the time zone database does not know |
| `index-fts5-missing` | the SQLite calendula links lacks FTS5 or is older than 3.43 |
| `query-refused` | the pimdir search query is malformed or names an unknown field or value |
| `query-unanswerable` | the pimdir search index could not answer an accepted query |
| `usage-invalid` | the command line does not match the grammar (an unknown flag or command, a missing argument, a bad value), raised by pimalaya-cli; nothing ran |

#### Scenario: A stale version
- GIVEN a pimdir event whose body changed since it was read
- WHEN `calendula --json event delete --if-match <old etag>` names it
- THEN the command exits 1, queues nothing and prints `{"code":"precondition-failed","error":"Precondition failed: …","sources":[],"backtrace":null}`

#### Scenario: An unknown item
- GIVEN the vdir calendar `personal` holding no item `nope`
- WHEN `calendula --json item read nope` runs
- THEN the command exits 1 and prints a failure object coded `item-not-found`

### Requirement: The JSON contract is described and versioned
`json-schema` SHALL describe the failure object under the key `calendula-error`, its `code` typed by the code vocabulary or `usage-invalid`, the code pimalaya-cli raises itself, and the item error of an `errors` list under `calendula-item-error`, its `code` typed by the code vocabulary. Every generated schema SHALL carry an `$id` naming the contract version: the major version of calendula, or `0.<minor>` before 1.0. Every registered payload SHALL be a JSON object, checked by a test. Piped `--json` output SHALL print one value per line.

### Requirement: A refused command line exits 2
The command line SHALL be parsed by pimalaya-cli's `clap::parse`: help and version print as clap prints them, and a command line clap refuses (an unknown flag or command, a missing argument, a bad value) SHALL exit 2 before any configuration is read. With a `--json` word before any `--`, it SHALL print on stdout the failure object coded `usage-invalid`, its `error` clap's message without its `error: ` prefix nor colour, and nothing on stderr; otherwise clap's rendering on stderr and nothing on stdout. Exit 1 stays a failure object and 3 a printed result that is not a full success: `account check` with a failed backend (see "account check fails when a backend fails") and a listing whose `errors` is not empty.

#### Scenario: A misspelt flag
- WHEN `calendula --json --nope calendar list` runs
- THEN it exits 2 and prints `{"code":"usage-invalid","error":"unexpected argument '--nope' found …","sources":[],"backtrace":null}` on one line
- AND `calendula --nope calendar list` exits 2, printing clap's `error: unexpected argument '--nope' found` on stderr and nothing on stdout

### Requirement: A listing names the items it could not read
`calendar list`, `event list`, `event search`, `event agenda`, `item list` and `pimdir search` SHALL print an `errors` list beside their items, under `--json` and in text: one pimalaya-cli item error `{id, code, error, sources}` per calendar or item the command could not read, `[]` when there is none, in the order met. `id` SHALL be the item's id as the command prints ids, `null` when the backend named none, as for a listing the server truncated; `code` a code of the vocabulary or `null`; `error` the message; `sources` the backend's own reasons, outermost first. A non-empty `errors` SHALL make the command exit 3 once it printed its result. A listing that reads the whole calendar before cutting its page SHALL print its entries on the first page only; an item it listed but could not read (an iCalendar that does not parse, a pimdir body the store cannot serve) holds its place on a page, and its entry SHALL print on that page. `event list` SHALL count the events it lists, never an item it could not read.

#### Scenario: An item that does not parse
- GIVEN the vdir calendar `personal` holding the events `a` and `b` and one `.ics` file `c` that is no iCalendar
- WHEN `calendula --json event list` runs
- THEN it prints `a` and `b`, `"errors":[{"id":"c","code":"item-unreadable",…}]`, and the exit status is 3

### Requirement: A command's errors print at its end
In the text output, each `errors` entry SHALL print as one `Error:` line, `<id>  <code>  <error>` with `-` for a null id or code, last in the command's output: after the items (rows, records or the agenda's lines) and one blank line, the `Error:` lines aligned among themselves as pimalaya-cli's `Columns` aligns cells, each value sanitized (pimalaya-cli `error::ItemErrors`). Nothing SHALL print, the blank line included, when `errors` is empty.

### Requirement: Calendar selection
A unified command operating inside a calendar SHALL take it through the flattened `-k/--calendar` flag, resolved by the account: the flag wins, otherwise `calendar.default`, otherwise the command SHALL fail coded `calendar-default-missing`, ``No default calendar found, name one with `-k <CALENDAR-ID>` or set `calendar.default` ``, as a pimdir command naming no calendar does.

A command acting on the calendar itself SHALL take it as an argument, as Himalaya's `mailbox delete <MAILBOX>` does, the former `-k/--calendar` kept as a hidden alias: `calendar update [CALENDAR]` resolves it as above, and `calendar delete <CALENDAR>` SHALL require it, never falling back to a default. `calendar create <NAME> [--id <ID>]` SHALL take the display name, and `--id` on the backends letting a client name a calendar, the former id before the name refused (exit 2): `--id` reaches `calendar.create.id`, `full` on vdir and CalDAV, `none` on Google, Graph and pimdir, refused before connecting; given none, vdir SHALL name the directory after the name, refused unless it is one directory name, and CalDAV SHALL mint a version 4 UUID for the URL segment, a display name holding what a path cannot; Google and Graph mint their own. `calendar create` and `calendar update` SHALL take `-d/--description` and `-C/--color`, as cardamum's `addressbook` commands do.

#### Scenario: A calendar named alone
- GIVEN a vdir account whose home holds no calendar
- WHEN `calendula --json calendar create Work`, then `calendar update Work -n Job -C '#ff0000'`, then `calendar delete Work` run
- THEN the first prints `{"id":"Work","name":"Work","queued":false}` and makes the directory `Work`, the second renames and paints it, the third removes it
- AND `calendar delete -k Work` is read as before, `calendar create Crew --id crew` makes the directory `crew`, and `calendar create crew Crew` exits 2 coded `usage-invalid`

### Requirement: A calendar update clears what it empties
`calendar update -d ""` and `-C ""` SHALL clear the description and the colour on every backend serving them, never report success leaving them: a CalDAV `DAV:remove` of the property, an empty description sent to Google, which stores it as none, vdir clearing its own. A Google calendar always carries a colour, so clearing one stays refused by name.

#### Scenario: A description cleared on CalDAV
- GIVEN a CalDAV calendar `cal` described `Old`
- WHEN `calendula calendar update cal -d ""` runs
- THEN one PROPPATCH removes `C:calendar-description`, and the next `calendar list --json` reads its `description` `null`

### Requirement: Nested execute
Each subcommand SHALL be a clap-derived struct carrying its own arguments, with an `execute(self, printer, client)` method, a unified command borrowing the client (`&mut CalendarClient`).

`Command::execute(printer, session)` SHALL be the single dispatch point: a unified command runs over the session's unified client, built on first use from the configuration and the account the session names (cairn/spec/repl.md), and a protocol-specific command builds its own client and hands it over.

### Requirement: Output goes to stdout
All data and errors SHALL go to stdout through the printer, with `--json` switching every command to JSON; stderr SHALL carry the logs, the prompts and the welcome only, plus the REPL's help in a JSON session and the failures of the MCP server, whose stdout carries protocol messages only. A command SHALL return a `Serialize + Display` value to the printer rather than printing inline.

### Requirement: Text output is plain text
A text output SHALL be plain text: no border, no colour, no terminal escape, nothing sized to or truncated for the terminal, and it SHALL end with one newline; no command takes a width. Every string a server, a store or a file gave (an id, a name, a summary, a description, an error a record carries, the zone and the days the lines under `pimdir search` and `pimdir count` name) SHALL have its control and bidi characters replaced before it prints as text (pimalaya-cli `sanitize::text`); the JSON output SHALL carry it unchanged. The iCalendar `event read`, `item read`, `caldav get` and `vdir item get` print SHALL keep its layout, CRLF read as LF, and have every other control character but tab and line feed (C0, DEL, C1) and every bidi embedding, override and isolate replaced with U+FFFD; their JSON `contents` SHALL carry the stored bytes unchanged, as text or as base64 ("An item's bytes are text or base64"), and their text view SHALL read bytes that are not valid UTF-8 as U+FFFD. The iCalendar a `build` prints, or writes to the file `-o` names, SHALL be the bytes it built.

`event list` and `event search` SHALL print as Himalaya's `envelope list` does, through pimalaya-cli's `listing::Listing`. Above the events, `Account:` (the account name, then the backend it resolved to in parentheses), `Calendar:` and, for `event search`, `UID:`, then `Page:` (the page number, `all` for an unpaged listing, then the events the page holds, `event N` or `events N to M` followed by `, the last` when no page follows, or `empty`), these values starting two spaces past the longest label; then a blank line. Then one line per event through pimalaya-cli's `Columns` with no header row: its item id, its start and its end (Columns follow the kind), then its summary, last, `-` for an empty cell. While a page follows, `Next page:` and the `--page` reaching it SHALL print after the events and one blank line (pimalaya-cli 0.6.0, D27). Under them SHALL print only the command's errors ("A command's errors print at its end"), after one more blank line. The events `Page:` counts are the calendar's events, the resources a page holds: a series counts once, however many lines its overrides or its occurrences in a window print. `, the last` and `Next page:` SHALL be exact on every backend: the unified client asks for one item past the page. `-s 0` lists every item, unpaged.

`item list` SHALL print the same header above one line per item: its id first, as every listing of the family, then its size and its entity tag. `calendar list` SHALL print as Himalaya's `mailbox list` does: `Account:` alone above one line per calendar, its id, `default` for the one the server names so, its color, then its name, last; its description prints under `--json` alone. `pimdir queue list` SHALL print as Himalaya's and cardamum's do: `Account:` (the account name and `pimdir`), `Calendar:` (the calendar listed, `all` for every one) and, while it holds a row, `Cancel:` and the command cancelling one, above one line per row, its queue row handle, `queued`, `parked` or `unconfirmed`, when it was queued, its kind, its calendar, then its summary (the event an intent addresses when the store holds no summary) or why it was parked; `Nothing queued nor parked` when it holds none. `gcal events instances` SHALL print it above one line per instance: its id, its status, its start and end as the server spells them, then its summary, `Event:` naming the series in the header, unpaged.

A list whose items carry more than one value of unbounded length SHALL print through pimalaya-cli's `Records`, the way `git log` prints commits: per item, its id alone on a line, followed by its state words in parentheses, comma-separated, when it has any; its properties as `Fields` prints them; its free text after a blank line, each line indented by four spaces; one blank line between items. A field every item carries SHALL print even when empty, its label alone; a field only some items carry SHALL be left out where it is empty. These lists print as records:

- `caldav propfind` with no calendar: the collection id; `Name` and, when set, `Color`, `Ctag`, `Sync token` and `Components`; the description.
- `msgraph calendars list`: the calendar id; `default` for the default one; `Name` and, when set, `Color` and `Owner`.
- `msgraph events list` and `msgraph events delta`: the event id, `changed` or `removed` for a delta; `Subject`, `Start`, `End`, `Location` and `Organizer`, those it holds; a removed event its id alone.
- `gcal calendars`: the calendar id; `primary`; `Name` and, when set, `Role`, `Time zone`, `Color` and `Reminders`; the description.
- `gcal free-busy`: the calendar id; `busy` or `free`, neither when the answer failed; `Busy`, one `<start> to <end>` line per period, and `Errors`.
- `pimdir search`: the item id; `Start`, `End`, `Calendar` and, when a text term matched, `Matched`; the summary, then the snippet after a blank line. What the answer could not see prints after a blank line.

Every other list SHALL print through pimalaya-cli's `Columns`: a header of uppercase labels, then one item per line, the cells two spaces apart and padded to the widest of their column, an empty cell as `-`, the free-text column last, several values in one cell joined with `,`: `vdir list` (ID, PATH, COLOR, NAME), `vdir item list` (ID, KIND, SIZE, PATH), `caldav propfind` with a calendar (ID, ETAG), `caldav report query` and `multiget` (ID, ETAG, SUMMARY, the errors last), `caldav report sync` (STATUS, ETAG, HREF), `caldav report raw` (STATUS, DATA-BYTES, ETAG, HREF), `gcal acl list` (ID, SCOPE, ROLE, VALUE), `gcal colors` (PALETTE, ID, BACKGROUND, FOREGROUND), and the calendars (ID, DOWNLOADED, QUEUED, NAME) and parked actions (ROW, KIND, CALENDAR, ERROR) of `pimdir status`.

`account list` SHALL print pimalaya-cli's `account::AccountListOutput` text, as Himalaya does: one line per account through `Columns` with no header row, its name, its backends (each the build leaves out as `<name> (not built)`, comma-separated), then `default` for the account a command with no `-a` runs against, `-` otherwise.

A record SHALL print through pimalaya-cli's `Fields`: one `Label: value` line per field, the values aligned past the longest label: `account check` (pimalaya-cli's `account::AccountCheckOutput`: `Account:`, then one `<backend>:` line, `OK` or `FAIL (<why>)`), `caldav discover` (`Server`, `Principal`, `Calendar home-set`, `(unresolved)` for one it could not resolve), `msgraph calendars get`, `create` and `rename`, `msgraph events get`, `create` and `update`, `msgraph profile get`, `gcal settings` (one field per setting) and the head of `pimdir status` (`Reading account:`, `Grouped accounts:`).

`msgraph request` SHALL print the JSON Graph answered, pretty, and nothing for a body Graph did not send (a 204). `gcal request` SHALL print the JSON Google answered, pretty, an empty body reading `{}` as io-gcal reads it. `gcal request` and `msgraph request` SHALL refuse a full URL off the API's origin (`https://www.googleapis.com`, `https://graph.microsoft.com`) before anything is sent, so the bearer token never leaves for another host.

The `event agenda` grid SHALL stay cal's drawing, with no attribute: no day is highlighted, the events under it naming their days.

#### Scenario: An event list
- GIVEN the vdir account `perso` whose calendar `personal` holds the timed events `a`, `b` and `c`, read in UTC
- WHEN `calendula event list -s 2` prints them
- THEN it prints `Account:   perso (vdir)`, `Calendar:  personal` and `Page:      1, events 1 to 2`, then a blank line, then `a  2026-11-12 09:00  2026-11-12 09:30  Standup` and the line of `b`, aligned, with no border and no colour, then a blank line and `Next page: --page 2`
- AND `calendula event list -s 2 -p 2` prints `Page:       2, event 3, the last` above the line of `c`

#### Scenario: A read of a body carrying controls
- GIVEN a vdir event stored with CRLF line ends, `ESC ]52;c;AAAA BEL` in its summary, U+202E in its location and a description folded on a tab
- WHEN `calendula event read` or `calendula item read` prints it
- THEN its lines end in LF, the ESC, the BEL and the U+202E print as U+FFFD, and the folded line keeps its tab
- AND under `--json` its `contents` is the stored text, CRLF and controls included

#### Scenario: A calendar list
- GIVEN the vdir account `perso` holding the calendars `personal`, the default, colored and described, and `work`
- WHEN `calendula calendar list` prints them
- THEN it prints `Account:    perso (vdir)`, a blank line, then `personal  default  #ff0000  Personal` and `work      -        -        Work`, aligned, the description left out

### Requirement: A page names the next one
`event list` and `item list` SHALL print `nextPage` beside their rows: the number of the next page, as a string, while one follows, `null` otherwise, exact on every backend as the text's `Next page:` is. They SHALL take it back through pimalaya-cli's `--page-token`, which wins over `--page`, refusing a token that does not read as a page number before the backend is reached, `0` reading as 1 as `--page 0` does. An unpaged listing (`-s 0`, or `event list` over a window with no `-s`) SHALL ignore the token as it ignores `--page`. `event search` SHALL page the same way: it reads the calendar whole, then cuts its matches by `-s` and `-p` or the token, a page counting resources as `event list` does; with no `-s` every match is one page, `nextPage` `null`.

#### Scenario: A listing resumed from its token
- GIVEN the vdir account `perso` whose calendar `personal` holds the events `a`, `b` and `c`
- WHEN `calendula --json event list -s 2` runs, then `calendula --json event list -s 2 --page-token 2`
- THEN the first prints `a` and `b` and `"nextPage":"2"`, the second `c` and `"nextPage":null`
- AND `calendula event list -s 2 -p 1 --page-token 2` prints `c`, the token winning, and `--page-token x2` fails naming `x2`

### Requirement: Help is plain text
`--help` and `-h` SHALL print without colour, each paragraph on one line, never wrapped to the terminal width, the top-level help ending with the Pimalaya footer (where to report bugs, how to sponsor).

### Requirement: A query error is drawn without colour
The report `pimdir search` and `pimdir count` draw over a query they refuse SHALL carry no colour escape, on a terminal as in the `--json` error string.

### Requirement: A bare invocation shows the help
A bare `calendula` SHALL print the help when a configuration exists, a broken one included, its error left to the first command reading it. Without one it SHALL fail as a command loading the configuration does: the welcome, or the error. Nothing prompts.

### Requirement: account check fails when a backend fails
`account check` SHALL exercise every backend of the account `--backend` allows and print one verdict per backend: under `--json`, `ok`, the stable `code` of a failure (the code vocabulary of "A JSON error carries a stable code", `null` when the backend passed or its failure carries none) and the error of a failed one; the text shows no code. When any fails, it SHALL still print the report and exit 3, the family's partial outcome (`pimalaya_cli::error::PartialOutcome`), so a script reads the outcome from the exit code alone and which backends passed from the report; when all pass, it SHALL exit 0. It mirrors Himalaya's `account check`.

Under `auto`, a block of a backend the build leaves out SHALL be a failed verdict of that backend, after those of the built backends, coded `backend-not-built`, its error naming the cargo feature bringing it back, so the command exits 3. A named `--backend` reaches built backends alone.

#### Scenario: One backend of two fails
- GIVEN the account `test` whose vdir home exists and whose pimdir root does not
- WHEN `calendula account check` runs
- THEN it prints `Account:  test`, `vdir:     OK` and `pimdir:   FAIL (<why>)`, and exits 3
- AND `calendula --json account check` prints the same report as its result, the pimdir verdict carrying `"code":"store-uncreated"` and the vdir one `"code":null`, and exits 3, and `calendula --backend vdir account check` exits 0

### Requirement: An account lists what it serves
`account capability` SHALL print pimalaya-cli's `capability::AccountCapabilityOutput`, reading the configuration and the tables and connecting nowhere: `{"account", "capabilities": [{"name", "backend", "level", "detail"}]}`, one row per capability of the unified API, `backend` the backend the unified commands resolve to (the first the account configures that `--backend` allows, in the order `auto` tries them), `level` one of `full`, `partial`, `server` and `none`, `detail` what the level leaves out or `null` for a full one. `--backend <name>` SHALL read that backend of the account. When the account configures no backend `--backend` allows, every row SHALL read `backend` `null` and level `none`, and the command SHALL succeed. When the backend the unified commands resolve to is one the build leaves out, the command SHALL fail coded `backend-not-built` (exit 1), with the message a unified command gives (see "Backend selection order" in backends). The text SHALL be a pimalaya-cli `listing::Listing` headed by `Account: <name>`, then a blank line and one line per capability with no column header: the name, the backend, the level and the detail.

#### Scenario: A vdir account
- GIVEN the account `test` configuring a vdir home alone
- WHEN `calendula account capability` runs
- THEN it prints `Account:    test`, a blank line and fifteen rows naming `vdir`, each `full` but `item.update.if-match`, `item.remove.if-match`, `online-meeting` and `scheduling.send-updates`, each `none` with why

#### Scenario: A backend the account lacks
- GIVEN the account `test` configuring a vdir home alone
- WHEN `calendula --json --backend pimdir account capability` runs
- THEN every row has `"backend": null` and `"level": "none"`, and the exit status is 0

### Requirement: An account names the backends left out of the build
`account list` SHALL print, per account, `{"name", "default", "backends", "notBuilt"}`: `backends` the backends the account carries a block for and the build serves, `notBuilt` those it carries a block for and the build leaves out, each in the order `auto` tries them. The backends cell of its text SHALL list the first, then each of the second as `<name> (not built)`, comma-separated.

#### Scenario: A gcal block in a build without gcal
- GIVEN a build without `gcal` and the account `test` carrying a `gcal` block alone
- WHEN `calendula --json account list` runs
- THEN `test` reads `"backends": []` and `"notBuilt": ["gcal"]`, and the text row reads `gcal (not built)`

### Requirement: Help is the usage reference
Each command's doc comment SHALL be its help text: the first paragraph is what `-h` shows, and the full text, ending with the command's JSON output shape, is what `--help` shows.

`calendula <command> --help` is therefore the canonical usage reference for both humans and agents, which is why the README documents no per-command usage.

### Requirement: The help names what each backend serves
The `--help` of each unified leaf reaching a backend SHALL end with a `Backends:` footer generated from the tables: per capability the leaf reaches, flag-dependent ones included, the backends grouped by level, then each detail with the backends it applies to. A `build` reaches no backend and SHALL carry none, `-h` SHALL show none, and the top-level help SHALL keep the Pimalaya footer.

### Requirement: The help reads as the family's
`-b/--backend` SHALL read as Himalaya's, `Force a specific backend for cross-protocol commands.`, its help listing the possible values, `auto` then every backend calendula knows in the order `auto` tries them, built in or not, each with the backend it pins. A value none is spelled SHALL be refused by clap listing them (exit 2, `usage-invalid` under `--json`); a backend the build leaves out SHALL still be refused naming the cargo feature bringing it back. `account` SHALL read `Manage accounts defined in the TOML configuration file`, `account list` `List all accounts declared in the configuration`, and each protocol namespace `<API>-specific API`: `CalDAV-specific API`, `Microsoft Graph-specific API`, `Google Calendar-specific API`, `pimdir-specific API` and `vdir-specific API`.

#### Scenario: An unknown backend
- GIVEN any build
- WHEN `calendula -b nope account list` runs
- THEN it exits 2, clap's error listing `[possible values: auto, vdir, pimdir, caldav, gcal, msgraph]`

### Requirement: Every data command returns a named output type
A command answering with data SHALL hand the printer a `*Output` type deriving `Display`, `Serialize` and `JsonSchema`, named after its command path. A write with no payload of its own SHALL print a receipt rather than a sentence: a unified write `{id, queued}` (`WriteReceipt`), the id of the item acted on and whether a sync engine applies the write later, and a write in a protocol namespace `{id}` (`IdReceipt`), the id as it is after the write (a renamed collection's new one). The write arm of `event add`, `event update`, `item add` and `item update` SHALL print `{id, queued}` too, a queued create's `id` the link id it was staged under, and `calendar create` SHALL print `{id, name, queued}`, `name` the display name it was given. The sentence is the text form only.

A `create`, an `update` and a `build` answer data too: an abandoned edit is an outcome they have to report, so each SHALL return an untagged enum whose write arm serializes exactly as it would on its own, and whose abandoned arm carries nothing, as an empty object. A `build` writing to `-o` answers pimalaya-cli's `output::SavedOutput`, `{path, size}`, the file written and its size in bytes, having handed the data to it, its text `Saved <size> bytes to <path>`.

Every runnable command SHALL be described in the `json-schema` registry, a command printing one of two types through both (`OneOf`), which a test checks over the clap command tree.

Every key of such a payload SHALL be camelCase, the spelling of the wire formats calendula sits over and the one a jq path addresses without quoting. The configuration schema stays kebab-case, being a TOML document a person writes rather than machine surface.

### Requirement: An item is edited through a command, never through a pipe
A composer SHALL be a command the configuration names, spawned on the path of a temporary file holding the iCalendar, with stdin, stdout and stderr all inherited. calendula SHALL capture none of the command's streams: a composer that spawns an editor would otherwise hand it a pipe instead of the terminal, and the editor hangs or writes where nothing reads. The command edits the file in place, and calendula reads it back once the command exits.

The file SHALL be created afresh in the temporary directory (pimalaya-cli's `file::private_options`, `create_new`): on Unix readable by its owner alone whatever the umask, and never written through a file or a symbolic link another user planted at its path, which fails the command instead.

The path SHALL be appended as the command's last argument. For a shell line that means interpolating it into the line through pimalaya-cli's `shell::quote`, single-quoted on a POSIX shell and double-quoted on Windows, where the line runs under `cmd /C`, since a shell invoked as `sh -c <line> <path>` binds the path to `$0` rather than passing it on.

### Requirement: The composer's own exit is the decision
When the composer exits, what it left in the file SHALL settle the edit, and nothing SHALL be asked after it. A file that came back changed is the item, and the command writes it. A file the composer emptied, and one handed back byte for byte as it was given, are an edit given up on: nothing is written, and nothing failed. A non-zero exit status is a failure and SHALL be reported as one.

Those three outcomes SHALL be the whole protocol, so a composer owning its own save and discard is not second-guessed by a menu it cannot see, and a plain editor keeps the meaning its own quit already has.

Abandoning SHALL exit as a success. A `build` SHALL print nothing when abandoned; `create` and `update` SHALL say that nothing was written.

### Requirement: A composed item is checked before it is written
What the composer wrote SHALL be checked against the RFC 5545 contract through ical-rs's validator rather than a look at its first line. Reading is liberal and this is the strict half: a VCALENDAR missing its required PRODID, a VEVENT missing its UID or DTSTAMP, are caught here rather than by the server or by nobody.

A VEVENT SHALL also be refused when it carries no DTSTART and the calendar specifies no METHOD (RFC 5545 3.6.1). That requirement is a condition on the enclosing calendar rather than a property list, so ical-rs's per-component contract cannot state it; calendula SHALL state it, because it is the one thing a person writing an event by hand leaves out and because a server does worse than refuse it: SabreDAV denormalizes DTSTART into its index on write and answers HTTP 500.

The check SHALL NOT refuse what RFC 5545 allows: a `DATE` value and a `TZID` on `DTSTART`, `DTEND`, `DUE`, `RECURRENCE-ID`, `EXDATE` and `RDATE`, and the `CN`, `DIR`, `SENT-BY` and scheduling parameters on `ORGANIZER`. A whole-day, zoned or organized event is not invalid. Every finding of ical-rs's validator SHALL be reported, a duration outside RFC 5545 3.3.6 included.

An item that does not pass SHALL have its violations printed and SHALL offer to re-open the editor, defaulting to yes. This is not a menu: the only question is whether to fix it, and declining is an error rather than an abandon.

An iCalendar given on the command line SHALL NOT be checked, going to the backend as it was written: that is the promise the projections already make, calendula never rewriting bytes it was handed.

### Requirement: An edit is never lost
When a composed item cannot be written, whether the check was declined or the backend rejected it, the temporary file SHALL be kept and the error SHALL name it: `Cannot edit iCalendar <path>`, which says what failed and where the work is in one line.

An abandoned edit SHALL drop the file instead: an emptied one holds nothing, and an untouched one holds only what calendula put there.

### Requirement: A composer is not spawned under --json
`--json` SHALL refuse to run a composer rather than spawn one. The child inherits calendula's own stdout, where it would interleave with the JSON payload, and a consumer parsing that output has no terminal to edit in.

### Requirement: A source and the composer stack
`create` and `update` SHALL take an iCalendar source and `-i/--interactive` together: the source is the item to start from, and `-i` opens it in the composer. `build` is the same pipeline stopped before the write.

The composer SHALL be opt-in through `-i`, never opt-out, which is what keeps every verb scriptable. `-i` SHALL bail when no composer is configured rather than fall back to one. `--composer <COMMAND>` SHALL override the configured one for a single invocation, and SHALL require `-i`. The pair SHALL be one shared argument, spelled once for every command that takes it.

A `create` given no source SHALL mint the item it starts from rather than open an empty file, so no composer is asked to invent an identity. An `update` given no source SHALL start from the item the backend holds, and SHALL send the entity tag it read as `If-Match` unless `--if-match` names another: an edit that takes a minute must not silently overwrite a write that landed during it.

A command given neither a source nor `-i` has nothing to write and SHALL say so; the `event` family also takes the field flags, which count as something to write.

### Requirement: The composer is the authoring surface
The composer SHALL remain the complete authoring surface; the field flags cover the common fields only. A recurrence rule, and any alarm beyond one display alarm before the start, are where a flag stops being ergonomic, and the documentation SHALL say so, so their absence reads as the boundary it is.

### Requirement: A source, the field flags and the composer stack
`event build`, `event add` and `event update` SHALL take an iCalendar source, the field flags and `-i/--interactive` together, applied in that order: the source (or the minted seed, or for an update the item the backend holds) is the event to start from, each flag sets the property it names on its VEVENT, and `-i` opens the result in the composer. A command given no source, no field flag and no `-i` has nothing to write and SHALL say so.

A build or a create made from flags alone, with no source and no `-i`, SHALL be checked as a composed item is and refused naming the violations; a source given on the command line SHALL NOT be.

### Requirement: The field flags are a convenience, not the surface
The field flags SHALL cover the fields `event list` renders and the few every calendar carries, and SHALL NOT grow to match what a composer models. Recurrence (`RRULE`, `RDATE`, `EXDATE`, overrides), every alarm but the one `--alarm` writes (several alarms, one after the end or related to it, an absolute one, an EMAIL or AUDIO one), conference links (`CONFERENCE`) and hand-written `VTIMEZONE`s are out; `--online-meeting` asks a sync engine to create a meeting and writes no link. The documentation SHALL say the composer is the complete surface.

### Requirement: One display alarm can be set from a flag
`event build`, `event add` and `event update` SHALL take `--alarm <LEAD>`, `LEAD` being an RFC 5545 3.3.6 duration in any case, a leading `+` dropped, checked by ical-rs's validator, or a short form: a whole number followed by `m`, `h` or `d` (`15m` is `PT15M`, `1h` is `PT1H`, `2d` is `P2D`). A negative lead, or one that is neither, SHALL be refused naming both shapes.

`--alarm` SHALL drop every VALARM of the VEVENT and write one in place of the first, or after the VEVENT's last item when it carried none: `ACTION:DISPLAY`, `TRIGGER:-<LEAD>` and a `DESCRIPTION` (required for DISPLAY, RFC 5545 3.6.6) holding the summary the event ends up with, `Reminder` when it has none. Every other line SHALL keep its bytes.

#### Scenario: A create with an alarm
- GIVEN a CalDAV account
- WHEN `calendula event add --summary Stand-up --start 2026-10-19T09:00Z --end 2026-10-19T09:15Z --alarm 15m` runs
- THEN the event is written with `BEGIN:VALARM`, `ACTION:DISPLAY`, `TRIGGER:-PT15M`, `DESCRIPTION:Stand-up`, `END:VALARM` inside its VEVENT

#### Scenario: An update replacing two alarms
- GIVEN an event carrying a DISPLAY VALARM at `-PT10M` and an EMAIL VALARM at `-P1D`
- WHEN `calendula event update <ID> --alarm PT1H` runs
- THEN the event carries one VALARM, `TRIGGER:-PT1H`, where the first stood, and every other line keeps its bytes

### Requirement: Every alarm can be removed from a flag
`--no-alarm` SHALL drop every VALARM of the VEVENT and write nothing in its place, and SHALL conflict with `--alarm`. Removing from an event that carries no VALARM SHALL leave the event byte for byte. On a backend falling back to the calendar's default reminders when an event carries none (gcal), the event SHALL get those defaults, as the projection says.

### Requirement: A lead a backend cannot hold is refused before the write
`event add` and `event update` SHALL refuse an `--alarm` lead the account's backend cannot hold, by name and before reaching it, never rounding it: on gcal a lead that is not whole minutes or exceeds four weeks (40320 minutes); on msgraph a lead that is not whole minutes, or that names days or weeks, which the msgraph projection does not read as a reminder. `event build` reaches no backend and SHALL write any lead the grammar allows. Google and Graph hold a lead in minutes, so the alarm reads back as `-PT<n>M` (`--alarm 1h` as `-PT60M`).

#### Scenario: A lead Google refuses
- GIVEN a gcal account
- WHEN `calendula event add --summary Trip --alarm 30d` (or `--alarm P5W`) runs
- THEN the command fails naming the four-week ceiling, and nothing is written

### Requirement: A field flag sets its property and touches nothing else
Writing a field flag SHALL drop every instance of the property it names on the VEVENT and write the flag's own, a repeated flag writing one instance per value. `--add-attendee` and `--remove-attendee` are the exception: each SHALL touch the one `ATTENDEE` it names (see [A field flag adds or removes one attendee](#requirement-a-field-flag-adds-or-removes-one-attendee)). Every other line SHALL keep its bytes. A time flag SHALL take a date, a local time, a `Z` time, or a time prefixed by an IANA zone (`Europe/Paris:2026-10-19T09:00`); `--time-zone` names the zone of the times that name none, and on `event update` such a time is otherwise read in the event's own zone ("An update reads a zone-less time in the event's zone"). A zone a flag names SHALL arrive with its `VTIMEZONE`, minted once from the time-zone database. On an update, `--start` given without `--end` or `--duration` SHALL move the end with it, keeping the event's length. `--end` given without `--start` SHALL be held to the `DTSTART` the event carries: a date against a time, either way, and an end before that start on one timeline, SHALL be refused before anything is written.

A line a flag writes longer than 75 octets SHALL be folded (RFC 5545 3.1), never inside a character. `--duration` SHALL take an RFC 5545 3.3.6 duration in any case, a leading `+` dropped, checked by ical-rs's validator; a negative one SHALL be refused.

### Requirement: A field flag adds or removes one attendee
`event build`, `event add` and `event update` SHALL take `--add-attendee [NAME:]ADDRESS` and `--remove-attendee ADDRESS`, each repeatable and spelled as `--attendee` is. `--add-attendee` SHALL write one `ATTENDEE` with `PARTSTAT=NEEDS-ACTION` and `RSVP=TRUE`, and a `CN` when a name is given, after the event's last `ATTENDEE`, or where `--attendee` writes when the event has none. On `event build` and `event add` the two flags work on the source or the minted seed, as every field flag does. `--remove-attendee` SHALL drop the `ATTENDEE` carrying that address. Addresses SHALL be compared case-insensitively on the `mailto:` value.

An address added that the event already carries, an address removed that it does not, an address given to both flags, and either flag given with `--attendee` SHALL each be refused naming the address or the flags, before anything is written.

#### Scenario: A guest joins
- GIVEN an event whose attendees are Ann, who accepted, and Bob, who declined
- WHEN `calendula event update --add-attendee carol@example.org ID` runs
- THEN Carol is written after Bob with `PARTSTAT=NEEDS-ACTION` and `RSVP=TRUE`, and Ann's and Bob's lines keep their bytes

#### Scenario: A guest already there
- GIVEN an event carrying `ATTENDEE:mailto:Ann@Example.org`
- WHEN `calendula event update --add-attendee "Ann B:ann@example.org" ID` runs
- THEN the update is refused naming `ann@example.org`, and the event is untouched

### Requirement: A backend without floating times anchors the time flags locally
On a backend that cannot store a floating time (gcal, msgraph), `event add` and `event update` SHALL read a `--start` or `--end` naming no zone in the local zone (`TZ`, then the system's), as `--time-zone` naming that zone would, `TZID` and `VTIMEZONE` included. `--time-zone` SHALL win when given; a date, a `Z` time and a zone-prefixed time SHALL be left as they are. The other backends SHALL keep such a time floating. A local zone with no IANA name SHALL anchor nothing.

The local zone is the one calendula reads a floating time in, so the event lands at the wall-clock time `event list` will show it at.

#### Scenario: A create on a Google account
- GIVEN a gcal account and a system zone of `Europe/Madrid`
- WHEN `calendula event add --start 2026-10-19T09:00 --end 2026-10-19T10:00` runs
- THEN the event is written with `DTSTART;TZID=Europe/Madrid:20261019T090000` and its `VTIMEZONE`, and Google accepts it

### Requirement: An update reads a zone-less time in the event's zone
On `event update`, a `--start` or `--end` naming no zone SHALL be read in the zone the event's `DTSTART` names in its `TZID`, when the time-zone database knows it, so the event moves on its own clock rather than turning floating; `--time-zone` SHALL keep the last word, and an event whose start names no such zone SHALL keep the reading of "A backend without floating times anchors the time flags locally".

#### Scenario: A zoned event moved by a local time
- GIVEN an event `DTSTART;TZID=Europe/Paris:20261112T090000` ending at 10:00
- WHEN `calendula event update <id> --start 2026-11-26T11:00` runs
- THEN it is written `DTSTART;TZID=Europe/Paris:20261126T110000` and `DTEND;TZID=Europe/Paris:20261126T120000`

### Requirement: Flags apply to one event
A source holding several VEVENTs SHALL be refused when a field flag is set, since a flag rewrites one event and the others would be dropped or left inconsistent. With no flag the source passes through as written.

### Requirement: An empty text flag removes its property
An empty value given to `--summary`, `--description`, `--location`, `--url`, `--categories`, `--organizer` or `--attendee` SHALL remove that property from the VEVENT rather than write it empty: every instance is dropped and nothing is written in its place. A repeated flag drops its empty values, so `--categories ""` or `--attendee ""` alone removes every `CATEGORIES` or `ATTENDEE`. Removing a property the event does not carry SHALL leave the event byte for byte. The time flags, `--duration`, `--status`, `--transparency` and `--sequence` SHALL keep refusing an empty value.

### Requirement: An update revises DTSTAMP
`event update` SHALL set `DTSTAMP` to the time of the write, in UTC (`YYYYMMDDTHHMMSSZ`), and `LAST-MODIFIED` to the same instant when the event carries one, whatever it writes: field flags, a source, or what `-i` hands back. Every VEVENT of the item is revised, one lacking a `DTSTAMP` getting one after its last property; every other line SHALL keep its bytes. The revision SHALL belong to the shared update pipeline, so every backend stores an event revised alike.

### Requirement: An online meeting can be asked for
`--online-meeting` SHALL write `X-PIMDIR-ONLINE-MEETING:TRUE` on the VEVENT (pimdir STORAGE Annex B.1), replacing any instance it carried. On pimdir, which declares `online-meeting` `partial`, the write SHALL be refused before it is queued when no source declares `calendar.online-meeting` (see the backends capability); every other backend declares it `none`, so `event add` and `event update` SHALL refuse the flag there with `capability-missing`, before reaching the backend. `event build` reaches no backend and SHALL simply write the property.

### Requirement: A gcal write chooses who is mailed
`event add`, `event update` and `event delete` SHALL take `--send-updates all|external-only|none`, which the gcal backend SHALL pass as the API's `sendUpdates` on the request it makes. Absent, update and delete SHALL omit `sendUpdates` and create SHALL choose as the Google Calendar backend requirement says, so nothing changes for a caller that does not give it.

On `event add` the flag SHALL override that choice wherever the event is inserted. An event that is imported instead mails nobody: `none` SHALL pass, and `all` or `external-only` SHALL be refused, naming the event's `ORGANIZER`, which gcal's `partial` `scheduling.send-updates` says. Every other backend declares `scheduling.send-updates` `none`, so the three commands SHALL refuse the flag there with `capability-missing`, before reaching the backend. `event build` SHALL NOT take it.

#### Scenario: A quiet fix
- GIVEN a gcal event with an attendee
- WHEN `calendula event update --summary "Typo fixed" --send-updates none ID` runs
- THEN the event is updated with `sendUpdates=none`, and Google mails nobody

#### Scenario: An invitation stored for later
- GIVEN a gcal account
- WHEN `calendula event add --attendee ann@example.org --send-updates none …` runs
- THEN the event is inserted with `sendUpdates=none` and its `iCalUID`, and Ann is not mailed

#### Scenario: Another backend
- GIVEN a CalDAV account
- WHEN `calendula event delete --send-updates none ID` runs
- THEN the command is refused with `capability-missing` naming `scheduling.send-updates` and `--send-updates`, and nothing is deleted

### Requirement: A minted item carries an identity, and its family names its kind
An item minted from nothing SHALL be a VCALENDAR carrying a PRODID and one VEVENT, carrying a UID minted as a fresh UUID, a DTSTAMP of now and a DTSTART of now, so the seed passes the check it will be held to the moment the composer hands it back. The values are placeholders, and an event with no time is not an event. An editor handed an empty file mints no identity, and the CalDAV resource name and the pimdir link id both derive from the UID.

`item` names no component kind, so it SHALL mint nothing: `item build` and `item add` given no source SHALL bail, naming `calendula event` for an event, `unfog` for a to-do and `journal` for a journal entry.

### Requirement: An item can be built without an account
`build` SHALL take the same iCalendar source and the same composer as `create`, apply them in the same order, and print the resulting iCalendar rather than sending it anywhere. It SHALL reach no backend.

It SHALL read no configuration, unless `-i` is given without `--composer` and the configured composer is the one thing it needs. An iCalendar can therefore be composed on a machine holding no configuration at all, and the account a family's subcommand runs against SHALL be resolved by that subcommand rather than ahead of the whole family.

`-o/--output <PATH>` SHALL write the item to that file instead of printing it, answering a message rather than the item. The flag earns its place on this command alone: the composer inherits stdout, so `event build -i > event.ics` hands the editor the file as its terminal, and with `-i` there is no redirection to fall back on.

### Requirement: An event build answers its projection
`event build` under `--json` SHALL answer `{contents, encoding, method, events}`: the iCalendar it built, `contents` and `encoding` as a read prints them ("An item's bytes are text or base64"), the calendar's `METHOD` uppercased (`null` when it carries none), and every VEVENT projected as `event read` projects them, the item id empty and the etag `null`. Its text output SHALL stay the iCalendar alone, a built item that is not UTF-8 printing as its bytes, as cardamum's `card build` does. `item build` SHALL answer `{contents, encoding}` the same way.

### Requirement: A source carries an item or it is refused
A source that holds nothing but whitespace SHALL be refused, naming where it was read from, rather than read as an item. Handing a backend an empty body is never what was meant, and an abandoned `build -i` prints nothing, which pipes straight into the next command.

### Requirement: The output schema is published
`calendula json-schema` SHALL print the JSON Schema of a named command's `--json` payload, or write one file per command under `--dir`. The registry SHALL be gated by the same cargo features as the commands it names, so it stays coherent under any feature combination.

### Requirement: pimdir searches calendar items through the store's index
`calendula pimdir search <query>...`, gated on the `pimdir` feature, SHALL answer the pimdir SEARCH query language (§8) from the store's search index, scoped to the account's events: the query is read as its conjuncts plus `kind:event`, `account:<account>` when the store groups the account under a name, and `collection:<id>` when `-k/--calendar` names a calendar, each a top-level conjunct. A group standing as a conjunct of the query given is read through under that scope (SEARCH §8.2), so `(collection:<id>) budget` has the scope and the coverage of `collection:<id> budget`. Without `-k` it SHALL search every calendar of the account, `calendar.default` not applied; a `-k` naming no calendar of the account SHALL be refused, naming those it holds. The words of the query are joined by spaces into one query. An option of `pimdir search` or `pimdir count` SHALL be read as the option wherever it stands, before the query or after it, `--json` included; a query word starting with `-`, a negated term, SHALL pass after `--` alone, a word starting with `-` before it being refused as an unknown option. Calendula SHALL open the index read-only (io-pimdir `PimdirIndex::open_query`) and SHALL NOT create, rebuild nor refresh it: the store's owner, its sync engine, does. The answer SHALL be the store as of the owner's last refresh: a queued write is not listed nor folded in.

A `date:` term SHALL compare the occurrences the index expanded (SEARCH §7), its days, `today` and `yesterday` read in the zone `event list` reads times in (`TZ`, then the system's), stated to the index as an IANA name (SEARCH §8.1); a zone with no IANA name SHALL read them in UTC, with a warning.

Each hit SHALL print on one line through pimalaya-cli's `Listing`: its id, the public `seq`, which `event read` takes; its start and end as `event list` shows them (Columns follow the kind), from its summary when its body is not local; its calendar; what a text term matched (`body` or `summary`), a column present when any hit of the page says; then its summary. The header SHALL name the account (`<name> (pimdir)`), the calendar (`-k`, else `all`), the query and the page, then the `Incomplete:` and `Zones:` fields when they apply; the next page (`Next page: --page <n>`) SHALL print after the hits. `--sort date|name|relevance` orders the hits, `date` (latest first) by default, a series by its first occurrence, and `--page` and `--page-size` page them, the page size defaulting to `event list`'s; pimalaya-cli's `--page-token`, the opaque cursor a page printed (io-pimdir `PimdirSearchCursor`, SEARCH §8.2: the place of its last hit, so a hit arriving or leaving between two pages neither repeats nor hides another), SHALL win over `--page`, a token that does not read as one refused, naming it; `--page <n>` SHALL walk the cursors of the pages before it, a page past the last answering no hit. Under `--json` the page carries `hits`, `nextPage` (the cursor of the next page, `null` on the last page), `coverage` and `errors`, each hit the event projection plus `calendarId`, `matched` (`null` when only field terms matched) and `snippet`. A hit the store cannot read (its row, its body, a hit no calendar holds) SHALL be an `errors` entry, its id the `seq`, coded by the store's refusal, else `item-unreadable`, ``Read item `<seq>` ``, the other hits listed, and the command SHALL exit 3.

#### Scenario: A search resumed from its token
- GIVEN an indexed pimdir calendar holding three events naming the budget
- WHEN `calendula --json pimdir search -s 2 budget` runs, then `calendula --json pimdir search -s 2 --page-token <nextPage> budget` with the `nextPage` the first printed
- THEN the first prints two hits and a `nextPage`, the second one hit and `"nextPage":null`, neither printing `more`, and `--page 2` prints the second page's hit
- AND `--page-token 0` or `--page-token two` fails naming the token

#### Scenario: An option after the query
- GIVEN a pimdir account
- WHEN `calendula pimdir search budget --page-size 3 --json` runs
- THEN it searches `budget`, three hits a page, and prints JSON
- AND `calendula pimdir search -- budget -lisbon` searches `budget -lisbon`

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

### Requirement: pimdir counts calendar items from the index
`calendula pimdir count <query>...` SHALL answer how many items the scoped query matches, with its coverage, under the same scope as `pimdir search`.

### Requirement: A pimdir calendar search says what it did not cover
The output of `pimdir search` and `pimdir count` SHALL carry the coverage of the answer (SEARCH §8.2): `bodiless`, the items in scope whose body is not local, matched on their summary alone; `missing`, those whose body the index could not read, and `unparseable`, those whose body it could not parse, both indexed from their summary; the three counted whatever the query reads; `unindexed`, those whose last change the index has not folded in; `behind`, how many changes the store drew above the index, store-wide; `since` and `until`, the range below or above which the store is not known to hold every item; `filling`, whether a first round is still listing a calendar in scope; `horizonLeft`, whether the date range reaches outside the occurrences the index holds; `cut`, how many series stop their occurrences before the range's end; `zoneDiffers`, whether a date term reads its days in another zone than the index read floating times and dates in; and `complete`, io-pimdir's own judgement of SEARCH §8.2 (`PimdirSearchCoverage::is_complete`), the lag included: false when any of these holds, but for `zoneDiffers`, which leaves an answer complete, and for a `bodiless`, a `missing` or an `unparseable` count while no term of the query reads a calendar body (a text term, `date:`, `when:`, `has:attachment`). The members SHALL be flat and camelCase. The plain output of `pimdir search` SHALL print an `Incomplete:` field above its hits, and `pimdir count` one `Incomplete:` line after its count, a blank line apart, naming what left the answer incomplete, a `bodiless`, a `missing` or an `unparseable` count only while io-pimdir counts it against the answer (`PimdirSearchReads::kinds` and, for `bodiless`, `counts_bodiless`), and nothing when the answer is complete.

#### Scenario: A body the index could not read
- GIVEN an index recording one event's body as missing
- WHEN `calendula pimdir search budget` runs
- THEN the coverage reports `missing` 1 and `complete` false, and the plain output says one item has a body the index could not read

#### Scenario: A query reading no body
- GIVEN the same index
- WHEN `calendula pimdir search organizer:alice@example.org` runs
- THEN the coverage reports `missing` 1 and `complete` true, and the plain output prints no coverage line

#### Scenario: An event not downloaded
- GIVEN an index over a calendar listing one event whose body is not local
- WHEN `calendula pimdir search date:2026-11` runs, then `calendula pimdir search organizer:alice@example.org`
- THEN the first reports `bodiless` 1 and `complete` false, its plain output saying one item has no local body, and the second reports `bodiless` 1 and `complete` true, its plain output printing no coverage line

#### Scenario: A range past the horizon
- GIVEN an index holding occurrences two years forward
- WHEN `calendula pimdir search date:2030` runs
- THEN the coverage reports `horizonLeft` true and `complete` false

### Requirement: A pimdir search names an index read in another zone
When the coverage of `pimdir search` or `pimdir count` reports a differing zone (SEARCH §8.2: the query holds a date term that can reach a calendar item, and the index reads floating times and dates in another zone than the one the search reads days in, SEARCH §7, `floating_zone`), the plain output SHALL say so, naming both zones, in a `Zones:` field above the hits of `pimdir search` and in a line after the count of `pimdir count`, and the JSON coverage SHALL carry `zoneDiffers` true. A query of no date term reads no day: it SHALL print no such line and carry `zoneDiffers` false.

#### Scenario: An index read in another zone
- GIVEN `TZ=Europe/Paris` and an index reading floating times in `UTC`
- WHEN `calendula pimdir search date:2026-11-07` runs
- THEN the plain output says the index reads floating times and dates in UTC and this search reads days in Europe/Paris, and the coverage reports `zoneDiffers` true and stays `complete`

#### Scenario: A query of no date term
- GIVEN `TZ=Europe/Paris` and an index reading floating times in `UTC`
- WHEN `calendula pimdir search budget` runs
- THEN the plain output names no zone and the coverage reports `zoneDiffers` false

### Requirement: A pimdir index or query failure is coded
A store with no search index SHALL fail both commands with the code `index-missing`, an index another version built with `index-outdated`, one of another draft or shape with `index-version`, one reading floating times, or a search reading days, in a zone the time zone database does not know with `index-zone-unknown`, and a SQLite lacking FTS5 with `index-fts5-missing`, each saying the store's sync engine builds or rebuilds the index and none advising to delete anything. A query the language refuses SHALL fail with `query-refused`: in plain output the SEARCH §8.4 fault drawn over the query, under `--json` the §8.4 code, span and expectations in `error`. An accepted query the index cannot answer SHALL fail with `query-unanswerable`.
