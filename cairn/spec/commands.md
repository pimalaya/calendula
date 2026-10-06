---
cairn: spec
capability: commands
status: current
---

# Commands

The command tree is split into three groups, in this order: the shared cross-protocol API, the protocol-specific escape hatches, and the meta commands. This is the standard Pimalaya CLI split, a portable surface plus per-protocol hatches.

### Requirement: The shared API is a strict least common denominator
The `calendar`, `event` and `item` families SHALL expose only operations every compiled backend can serve identically. A concept one backend has and another does not SHALL NOT appear here; it belongs to that backend's own family.

Adding a backend that cannot serve a shared operation SHALL move the operation out rather than have it emulated.

### Requirement: Protocol-specific families
`configure` (alias `wizard`) SHALL run the account wizard.

`caldav`, `gcal`, `pimdir` and `vdir` SHALL each expose what only that backend has, gated behind its own cargo feature. CalDAV covers `discover`, its own calendar listing (carrying the ctag, the sync token and the accepted component kinds), `create` and `delete`.

gcal covers the half of the Calendar API iCalendar cannot express: sharing, availability, recurrence expansion, server-side parsing and the palettes. vdir covers its collection verbs including `rename`, which the shared API has no home for.

pimdir covers `status`, reporting the account being read, every account the store groups collections under, how much of each calendar is downloaded, and how many creations are queued for the next sync; and `reply` and `cancel`, the scheduling intents only its sync engine can perform.

A backend MAY exist without a family where its protocol adds nothing the shared surface lacks; a family SHALL NOT push protocol-specific concepts into the shared API instead.

### Requirement: gcal family
`gcal` SHALL cover: `calendars`, the richer listing carrying the access role, the primary flag, the time zone and the default reminders a shared `Calendar` has no room for; `acl` with `list`, `create`, `update` and `delete` over the sharing rules; `free-busy`, the availability query over a window.

It SHALL also cover `instances`, the occurrences a recurring event expands to; `move`, relocating an event to another calendar; `quick-add`, an event parsed server-side from a sentence; `colors`, the two palettes the API's colour ids refer to; and `settings`, the user's own.

An ACL rule id is `<scope type>:<scope value>`, which the API mints, so `create` and `update` SHALL take the scope and the role as flags and derive it, and `delete` SHALL accept the id a listing showed.

`free-busy` and `instances` SHALL take `--from` / `--to` as inclusive days, as `event list` does, so one date spelling serves the whole CLI.

### Requirement: What the gcal family declines
`channels` / `watch` SHALL NOT be exposed: a push channel delivers to an HTTPS endpoint the caller must host, which a CLI has not. `calendars.transferOwnership` SHALL NOT be exposed: it is an irreversible administrative act on a Workspace domain.

`calendars.clear` SHALL NOT be exposed: it empties the primary calendar with no undo, and `calendar delete` covers the secondary case. These are declined on purpose, and recorded as such rather than left as gaps.

### Requirement: The component families against item
The shared API SHALL offer two kinds of view over the same resources. `item` is the raw, unfiltered one: it lists, reads, writes and deletes any iCalendar object by id, leaving the bytes untouched.

The component families (`event`, `todo`, `journal`) are the projected ones: each keeps its own component kind, renders the columns that kind is read by, and `event` additionally draws a cal(1)-style agenda.

All SHALL share the `-k/--calendar` selector and the same item API; only the rendering and the filter differ.

### Requirement: One family per stored component kind
The shared API SHALL offer one command family per iCalendar component kind a calendar stores: `event` for VEVENT, `todo` for VTODO and `journal` for VJOURNAL.

Each SHALL cover `list`, `read`, `create`, `update` and `delete` and read the same item API, so a component family costs a projection and a table and adds no backend operation. An item whose bytes do not parse SHALL project nothing rather than fail the listing.

### Requirement: Columns follow the kind
A component listing SHALL render the properties its kind is read by: an event its summary and its start and end, a todo its summary, due date, status, priority and completion percentage, a journal its summary and its date.

A property the component omits SHALL render empty rather than absent, so the columns line up down the table. A todo whose STATUS is `COMPLETED` SHALL read as fully done even where PERCENT-COMPLETE is absent, since RFC 5545 3.8.1.8 makes the one imply the other.

### Requirement: Only events draw an agenda
`agenda` SHALL stay a VEVENT command. The grid marks the days that carry an event because an event occupies time; a todo's due date and a journal's date do not fill a day, and drawing them the same way would say they do.

### Requirement: An agenda shows every event at an instant
The agenda SHALL render every event whose start falls at a given instant, never one of them, and SHALL order two events at one instant totally and stably, by label then by item id, so the same calendar renders the same way twice.

The events at one instant SHALL print under one time column rather than repeating it, so a duplicated pair reads as two events rather than as two rows that happen to share a time.

Two events may legitimately start at the same moment: two unrelated meetings at 09:00, and, since a collection may hold two resources under one `UID` (pimdir SPEC 9), two copies of what a server considers one identity.

A view keyed by the instant alone shows the last one written and reports nothing, which loses exactly the event the store went to the trouble of keeping.

The `--json` payload SHALL carry the same multiplicity, so a consumer reading it is not told there was one event where there were two: it maps each start datetime to the list of the labels starting at it, in the same order the text renders them.

No second shape SHALL be kept beside it, since one that still dropped an event would leave the defect reachable.

### Requirement: A page of a family holds that family
A listing SHALL narrow by component kind before it paginates. Filtering after paginating makes a page mean "the components of this kind among the first N items of any kind", which on a calendar dominated by another kind is empty however many matches the calendar holds.

The kind SHALL travel with the page and the window as one query, so no backend can serve one without the others. A backend SHALL push the kind down where its protocol defines such a filter: CalDAV as an RFC 4791 `comp-filter`, with the `time-range` nested inside it as 9.7.1 requires. A backend modelling one kind only SHALL answer any other kind without a round-trip.

The raw `item` family names no kind and SHALL keep every one, which is what makes it the unfiltered view.

A pimdir item whose body is not local SHALL answer its kind from the summary table the store filed it in (event, task or journal), which names the component a reader renders the resource as, so narrowing by kind keeps the listing availability-aware.

### Requirement: A component window is applied locally
`event list` pushes its window down where the backend can narrow server-side. `todo list` and `journal list` SHALL apply theirs after parsing instead.

A server-side range filter is defined against a component's start and end (RFC 4791 9.9), which a todo (due, no start) and a journal entry (dated, no end) do not both carry, so a pushed-down filter would drop them for the wrong reason.

A component carrying no date at all SHALL show only in an unfiltered listing.

### Requirement: Kinds with no family
VFREEBUSY and VTIMEZONE SHALL NOT get a family.

A VFREEBUSY is the answer to a query rather than a resource a calendar stores, and where a backend offers such a query it belongs to that backend's own protocol-specific family. A VTIMEZONE defines the zones the other components reference and is not an item a user lists.

### Requirement: Projections never rewrite bytes
Projecting a VEVENT out of an item SHALL be read-only and lossy by design: calendula returns what the backend holds. An item whose bytes do not parse SHALL project no event rather than fail the listing, so one malformed resource cannot hide a whole calendar.

"What the backend holds" is verbatim for the backends that store iCalendar, and synthesized for the one that does not; the [projection](./projection.md) capability governs the difference, and no command layer SHALL rewrite bytes of its own.

### Requirement: Time-range filtering
`event list` SHALL accept `--from` and `--to` as inclusive days. The pair SHALL map onto a range whose upper bound is exclusive (the day after, at midnight UTC), so `--to` covers the whole day named.

A crossed pair SHALL be rejected by name. A range SHALL lift the default page-size cap, so a window returns every match rather than its first page.

A backend filtering locally (vdir, msgraph, pimdir) SHALL keep an item when one of its events starts in the window or one of its occurrences overlaps it. An undownloaded pimdir series the summary says recurs SHALL pass from its `DTSTART` to its `UNTIL`.

### Requirement: A window expands recurring events
With a window, `event list` SHALL list every occurrence overlapping it rather than every VEVENT: the recurrence set of each `UID` (`DTSTART`, `RRULE`, `RDATE`, minus `EXDATE`, with its `RECURRENCE-ID` overrides) walked by ical-rs, an override standing in for the instance it names, and one moved into the window from an identity past it still listed. An override travelling without its series lists on its own.

An occurrence SHALL carry the item `id` and its `recurrenceId`, the instance identity spelled as the series' `DTSTART` (a date, a local time, or a `Z` time), which `event read -r` takes back. Occurrences SHALL list in start order. A timed occurrence overlaps when it starts before the window's end and ends after its start (a zero-length one when it starts inside); a whole-day one compares by date. A window open on its end SHALL reach a year past its start.

Without a window a series SHALL list once, at its first start.

### Requirement: Times resolve to instants
A time SHALL resolve through its `TZID` as the time-zone database names it (a writer's path prefix tolerated), else through the calendar's `VTIMEZONE` of that `TZID`, else in the local zone, as a floating time does; a `Z` time is UTC. A local time a transition skips SHALL read with the offset before the gap, one it repeats as its first occurrence (RFC 5545 3.3.5).

Expansion SHALL move a UTC `UNTIL` onto the wall clock of a zoned `DTSTART` before the civil walk, and SHALL drop the instances a calendar-defined zone skips.

### Requirement: The event projection
An event's JSON SHALL carry `id`, `etag` (the entity tag of its item, `null` when the backend has none), `uid`, `recurrenceId`, `summary`, `description`, `location`, `start` and `end` (iCalendar-spelled), `allDay`, `startsAt` and `endsAt` (RFC 3339 with offset, or `YYYY-MM-DD` with an exclusive end for a whole day), `timeZone`, `zoneAssumed` (`true` when `DTSTART` or `DTEND` names a `TZID` neither the time-zone database nor the calendar defines, read in the local zone), `recurring`, `sequence` (its `SEQUENCE`, 0 when absent), `status`, `transparency`, `busyStatus` (Outlook's), `organizer` (`email`, `name`), `attendees` (`email`, `name`, `partstat` defaulting to `NEEDS-ACTION`, `role`, `rsvp`, `cutype`) and `onlineMeetingUrl` (`CONFERENCE`, then the vendors' properties). The end SHALL be `DTEND`, else the start plus `DURATION`, else a day for a date and the start for a time.

`event read` SHALL print the item's bytes, and its JSON SHALL add the projected `events`, or the one occurrence `-r/--recurrence-id` names.

### Requirement: An event is found by its UID
`event find <UID>` SHALL list, unexpanded, every VEVENT of the calendar carrying that `UID`, with the id the other `event` commands take.

### Requirement: An event is deleted by its id or its UID
`event delete <EVENT-ID>` SHALL take the id `event list` reports or the event's iCalendar `UID`. It SHALL try the id first, by a read, then list the calendar's events and look for an item of that id, then for the items carrying that `UID`. One item SHALL be deleted whole; several carrying the `UID` SHALL be refused, naming their ids; none SHALL be a clear miss.

### Requirement: A delete can be gated on a version
`event delete`, `todo delete`, `journal delete` and `item delete` SHALL take `--if-match <ETAG>`. CalDAV SHALL send it as `If-Match`, msgraph SHALL compare it with the event's `changeKey`, and pimdir SHALL check it as an update does. A backend that cannot gate a delete (vdir, gcal) SHALL refuse the flag rather than drop it.

`item read` SHALL add the `etag` of the version read to its JSON, `null` when the backend has none.

### Requirement: A JSON error carries a stable code
A failure a caller is expected to act on SHALL carry a stable code, printed under `--json` as a `code` string beside `error`, `sources` and `backtrace`. The code SHALL be found anywhere in the error chain, so added context does not hide it. A failure with no code SHALL print no `code` field. The plain output SHALL stay unchanged. Codes are kebab-case and never renamed; `error` stays free wording.

| Code | Raised when |
|---|---|
| `body-pending` | a listed item's body is not local yet (pimdir); a sync brings it |
| `precondition-failed` | a write's `--if-match` names a version the item no longer has (pimdir) |
| `occurrence-unsupported` | `pimdir reply` or `pimdir cancel` names an occurrence its performer cannot act on alone |

#### Scenario: A stale version
- GIVEN a pimdir event whose body changed since it was read
- WHEN `calendula --json event delete --if-match <old etag>` names it
- THEN the command exits 1, queues nothing and prints `{"code":"precondition-failed","error":"Precondition failed: …","sources":[],"backtrace":null}`

### Requirement: Calendar selection
A shared command operating inside a calendar SHALL take it through the flattened `-k/--calendar` flag, resolved by the account: the flag wins, otherwise `calendar.default`, otherwise the command bails.

`calendar delete` is the one exception and SHALL inline a mandatory `-k/--calendar`, never falling back to a default.

### Requirement: Nested execute
Each subcommand SHALL be a clap-derived struct carrying its own arguments, with an `execute(self, printer, client)` method.

`CalendulaCommand::execute` SHALL be the single dispatch point: it loads the configuration, selects the account, builds the appropriate client and hands it over.

### Requirement: Output goes to stdout
All data and errors SHALL go to stdout through the printer, with `--json` switching every command to JSON; stderr SHALL carry logs and prompts only. A command SHALL return a `Serialize + Display` value to the printer rather than printing inline.

### Requirement: Help is the usage reference
Each command's doc comment SHALL be its help text: the first paragraph is what `-h` shows, and the full text, ending with the command's JSON output shape, is what `--help` shows.

`calendula <command> --help` is therefore the canonical usage reference for both humans and agents, which is why the README documents no per-command usage.

### Requirement: Every data command returns a named output type
A command answering with data SHALL hand the printer a `*Output` type deriving `Display`, `Serialize` and `JsonSchema`, named after its command path. `Message` carries a confirmation and never data.

A `create`, an `update` and a `build` answer data too: an abandoned edit is an outcome they have to report, so each SHALL return an untagged enum whose write arm serializes exactly as it would on its own, and whose abandoned arm carries nothing. A `build` writing to `-o` answers a `Message`, having handed the data to a file.

Every key of such a payload SHALL be camelCase, the spelling of the wire formats calendula sits over and the one a jq path addresses without quoting. The configuration schema stays kebab-case, being a TOML document a person writes rather than machine surface.

### Requirement: An item is edited through a command, never through a pipe
A composer SHALL be a command the configuration names, spawned on the path of a temporary file holding the iCalendar, with stdin, stdout and stderr all inherited. calendula SHALL capture none of the command's streams: a composer that spawns an editor would otherwise hand it a pipe instead of the terminal, and the editor hangs or writes where nothing reads. The command edits the file in place, and calendula reads it back once the command exits.

The path SHALL be appended as the command's last argument. For a shell line that means interpolating it into the line, single-quoted, since a shell invoked as `sh -c <line> <path>` binds the path to `$0` rather than passing it on.

### Requirement: The composer's own exit is the decision
When the composer exits, what it left in the file SHALL settle the edit, and nothing SHALL be asked after it. A file that came back changed is the item, and the command writes it. A file the composer emptied, and one handed back byte for byte as it was given, are an edit given up on: nothing is written, and nothing failed. A non-zero exit status is a failure and SHALL be reported as one.

Those three outcomes SHALL be the whole protocol, so a composer owning its own save and discard is not second-guessed by a menu it cannot see, and a plain editor keeps the meaning its own quit already has.

Abandoning SHALL exit as a success. A `build` SHALL print nothing when abandoned; `create` and `update` SHALL say that nothing was written.

### Requirement: A composed item is checked before it is written
What the composer wrote SHALL be checked against the RFC 5545 contract through ical-rs's validator rather than a look at its first line. Reading is liberal and this is the strict half: a VCALENDAR missing its required PRODID, a VEVENT missing its UID or DTSTAMP, are caught here rather than by the server or by nobody.

A VEVENT SHALL also be refused when it carries no DTSTART and the calendar specifies no METHOD (RFC 5545 3.6.1). That requirement is a condition on the enclosing calendar rather than a property list, so ical-rs's per-component contract cannot state it; calendula SHALL state it, because it is the one thing a person writing an event by hand leaves out and because a server does worse than refuse it: SabreDAV denormalizes DTSTART into its index on write and answers HTTP 500.

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

A command given neither a source nor `-i` has nothing to write and SHALL say so.

### Requirement: The composer is the authoring surface
calendula SHALL NOT grow per-property flags for the component families. A recurrence rule, a time zone reference and an attendee list are where a flag stops being ergonomic, and the documentation SHALL say the composer is the complete surface, so the absence reads as the boundary it is.

### Requirement: A minted item carries an identity, and its family names its kind
An item minted from nothing SHALL be a VCALENDAR carrying a PRODID and one component of the family's own kind, carrying a UID minted as a fresh UUID and a DTSTAMP of now. An editor handed an empty file mints no identity, and the CalDAV resource name and the pimdir link id both derive from the UID.

A minted VEVENT SHALL also carry a DTSTART of now, so the seed passes the check it will be held to the moment the composer hands it back. The value is a placeholder as the identity is, and an event with no time is not an event. A VTODO and a VJOURNAL owe none and SHALL carry none.

`item` names no component kind, so it SHALL mint nothing: `item build` and `item create` given no source SHALL bail, naming `event`, `todo` and `journal` as the families that can.

### Requirement: An item can be built without an account
`build` SHALL take the same iCalendar source and the same composer as `create`, apply them in the same order, and print the resulting iCalendar rather than sending it anywhere. It SHALL reach no backend.

It SHALL read no configuration, unless `-i` is given without `--composer` and the configured composer is the one thing it needs. An iCalendar can therefore be composed on a machine holding no configuration at all, and the account a family's subcommand runs against SHALL be resolved by that subcommand rather than ahead of the whole family.

`-o/--output <PATH>` SHALL write the item to that file instead of printing it, answering a message rather than the item. The flag earns its place on this command alone: the composer inherits stdout, so `event build -i > event.ics` hands the editor the file as its terminal, and with `-i` there is no redirection to fall back on.

### Requirement: An event build answers its projection
`event build` under `--json` SHALL answer `{contents, method, events}`: the iCalendar it built, the calendar's `METHOD` uppercased (`null` when it carries none), and every VEVENT projected as `event read` projects them, the item id empty and the etag `null`. Its text output SHALL stay the iCalendar alone. The other families' `build` SHALL keep answering `{contents}`.

### Requirement: A source carries an item or it is refused
A source that holds nothing but whitespace SHALL be refused, naming where it was read from, rather than read as an item. Handing a backend an empty body is never what was meant, and an abandoned `build -i` prints nothing, which pipes straight into the next command.

### Requirement: The output schema is published
`calendula json-schema` SHALL print the JSON Schema of a named command's `--json` payload, or write one file per command under `--dir`. The registry SHALL be gated by the same cargo features as the commands it names, so it stays coherent under any feature combination.
