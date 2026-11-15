---
cairn: delta
change: the-namespaces-follow-the-twin
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: A former command name stays hidden
(commands) A protocol command renamed or moved SHALL keep its former name, hidden from the help, the manual and the completions, so a script keeps working: an alias where the command keeps its depth (`caldav list` and `ls` for `caldav propfind`, `caldav create` for `caldav mkcalendar`), a hidden command running the same code where it moved deeper (`gcal instances` and `instance`, `gcal move` and `gcal quick-add` for `gcal events instances`, `move` and `quick-add`), each printing what its new name prints. A hidden command keeps its JSON schema key (`calendula-gcal-instances`); an alias has none of its own, its schema being its command's (`calendula-caldav-propfind`).

#### Scenario: A former name
- GIVEN a CalDAV account and a gcal one
- WHEN `calendula caldav list`, `calendula caldav create work` or `calendula gcal quick-add "Lunch tomorrow"` runs
- THEN it runs `caldav propfind`, `caldav mkcalendar work` or `gcal events quick-add "Lunch tomorrow"`, printing what that prints, and none of the three shows in `--help`

### Requirement: caldav speaks the protocol
(commands) `caldav` SHALL be the WebDAV and CalDAV methods (RFC 4918, 4791, 6578) run against the account's endpoint, each named after its method, as cardamum's `carddav` is: `discover`, `propfind`, `proppatch`, `mkcalendar`, `report` (`query`, `multiget`, `sync`, `raw`), `get`, `put` and `delete`. Each takes its calendar and item ids positionally, and refuses one no CalDAV resource can have before anything connects (backends, "A CalDAV id addresses its own resource").

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
(commands) `msgraph`, gated on the `msgraph` feature, SHALL be nested by Graph resource, each command named after its Graph operation and printing Graph's raw objects under `--json`, as cardamum's `msgraph` is: `calendars` (`list`, `get`, `create`, `rename`, `delete`), `events` (`list`, `get`, `create`, `update`, `delete`, `delta`), `profile get` (`/me`) and `request <METHOD> <PATH> [JSON]`, the escape hatch printing `{"response"}`. `calendars list` SHALL print `{"calendars": [...]}` and `events list` `{"events": [...]}`, one Graph page, `@odata.nextLink` beside when one follows; `events delta` one page of the calendar view delta over `--from` and `--to` (inclusive days, required without `--delta-link`), `{"events": [...]}` with the `@odata.nextLink` or `@odata.deltaLink` it ended on, which `--delta-link` takes back. `events create` and `update` SHALL take a raw Graph event JSON body (a file, inline, or `-` for stdin), `update` a PATCH; `-k/--calendar` names the calendar of `events list`, `create` and `delta`, the default one when omitted. A delete prints `{id}`. A request naming one event Graph does not hold, a 404 or a 400 `ErrorInvalidIdMalformed`, SHALL fail `item-not-found`; one naming a calendar so, `calendar-not-found`, Graph's message kept. The text prints a list as records, one item as fields.

### Requirement: vdir items
(commands) `vdir item` (visible alias `items`) SHALL cover the raw files of a collection, byte for byte and of any kind, as cardamum's `vdir item` does: `list <COLLECTION>` (`{"items": [{id, kind, size, path}]}`), `get <COLLECTION> <ID>` (`{id, kind, contents, encoding}`, its text the bytes), `create <COLLECTION> [--kind ical|vcard] <INPUT>` (the kind sniffed from the bytes when omitted, `{id}`), `update <COLLECTION> <ID> <INPUT>` (its kind kept), `delete`, `copy <SOURCE> <TARGET> <ID>` and `move <SOURCE> <TARGET> <ID>`, each write printing `{id}`. A collection that is not one directory name of the home SHALL fail `calendar-not-found`, an item it does not hold `item-not-found`.

## MODIFIED Requirements

### Requirement: Protocol-specific families
(commands) `caldav`, `msgraph`, `gcal`, `pimdir` and `vdir` SHALL each expose what only that backend has, gated behind its own cargo feature, shaped as cardamum's: CalDAV a flat list of WebDAV method names ("caldav speaks the protocol"), Microsoft Graph and Google Calendar nested by the API's resource, each resource named by the API's own spelling, plural included, with its singular as a hidden alias, vdir flat collection verbs plus an `item` subtree. `caldav mkcalendar` and `vdir create` SHALL take the calendar's metadata with the letters `calendar create` and `calendar update` use: `-n`, `-d/--description`, `-C/--color`, the former `-D` read as `-d`.

gcal covers the half of the Calendar API iCalendar cannot express: sharing, availability, recurrence expansion, server-side parsing and the palettes. vdir covers its collection verbs including `rename`, which the unified API has no home for, and its raw items.

(The pimdir paragraph and the last one stay.)

### Requirement: gcal family
(commands) `gcal` SHALL cover: `calendars`, the richer listing carrying the access role, the primary flag, the time zone and the default reminders a shared `Calendar` has no room for; `acl` with `list`, `create`, `update` and `delete` over the sharing rules; `free-busy`, the availability query over a window.

It SHALL also cover `events`, the Calendar API's Events resource (hidden alias `event`): `events instances`, the occurrences a recurring event expands to; `events move`, relocating an event to another calendar; `events quick-add`, an event parsed server-side from a sentence. And `colors`, the two palettes the API's colour ids refer to, and `settings`, the user's own. `gcal instances`, `move` and `quick-add` SHALL stay hidden commands running the same ("A former command name stays hidden").

`free-busy` and `events instances` SHALL take `--from` / `--to` as inclusive days, as `event list` does, so one date spelling serves the whole CLI.

(The paragraphs on unknown values and on ACL rule ids stay.)

### Requirement: Text output is plain text
(commands) `gcal instances` becomes `gcal events instances` wherever the requirement names it. The records list SHALL name `caldav propfind` (with no calendar) where it named `caldav list`, and add `msgraph calendars list` (the calendar id; `Name`, `Color`, `Default`, `Owner`), `msgraph events list` and `msgraph events delta` (the event id, `changed` or `removed` for a delta; `Subject`, `Start`, `End`, `Location`, `Organizer`, those it holds). The Columns list SHALL add `caldav propfind` with a calendar (ID, ETAG), `caldav report query` and `multiget` (ID, ETAG, SUMMARY), `caldav report sync` (STATUS, ETAG, HREF), `caldav report raw` (STATUS, DATA-BYTES, ETAG, HREF) and `vdir item list` (ID, KIND, SIZE, PATH). The Fields list SHALL add `msgraph calendars get`, `create` and `rename`, `msgraph events get`, `create` and `update`, and `msgraph profile get`. `caldav get` and `vdir item get` print the bytes as `event read` prints an iCalendar, and `msgraph request` the pretty JSON it answered, nothing for a 204.

### Requirement: A CalDAV id addresses its own resource
(backends) `caldav delete` becomes every `caldav` command naming a calendar or an item (`propfind`, `proppatch`, `report`, `get`, `put` and `delete`); `caldav create` becomes `caldav mkcalendar`.

## REMOVED Requirements
