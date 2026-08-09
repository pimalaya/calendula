---
cairn: spec
capability: commands
status: current
---

# Commands

The command tree is split into three groups, in this order: the shared cross-protocol API, the protocol-specific escape hatches, and the meta commands. This is the standard Pimalaya CLI split, a portable surface plus per-protocol hatches.

### Requirement: The shared API is a strict least common denominator
The `calendar`, `event` and `item` families SHALL expose only operations every compiled backend can serve identically. A concept one backend has and another does not SHALL NOT appear here; it belongs to that backend's own family. Adding a backend that cannot serve a shared operation SHALL move the operation out rather than have it emulated.

### Requirement: Protocol-specific families
`caldav`, `gcal`, `pimdir` and `vdir` SHALL each expose what only that backend has, gated behind its own cargo feature. CalDAV covers `discover`, its own calendar listing (carrying the ctag, the sync token and the accepted component kinds), `create` and `delete`. gcal covers the half of the Calendar API iCalendar cannot express: sharing, availability, recurrence expansion, server-side parsing and the palettes. vdir covers its collection verbs including `rename`, which the shared API has no home for. pimdir covers `status`, reporting the source writes are attributed to, every source the store has been synced as, and how much of each calendar is downloaded.

A backend MAY exist without a family where its protocol adds nothing the shared surface lacks; a family SHALL NOT push protocol-specific concepts into the shared API instead.

### Requirement: gcal family
`gcal` SHALL cover: `calendars`, the richer listing carrying the access role, the primary flag, the time zone and the default reminders a shared `Calendar` has no room for; `acl` with `list`, `create`, `update` and `delete` over the sharing rules; `free-busy`, the availability query over a window; `instances`, the occurrences a recurring event expands to; `move`, relocating an event to another calendar; `quick-add`, an event parsed server-side from a sentence; `colors`, the two palettes the API's colour ids refer to; and `settings`, the user's own.

An ACL rule id is `<scope type>:<scope value>`, which the API mints, so `create` and `update` SHALL take the scope and the role as flags and derive it, and `delete` SHALL accept the id a listing showed. `free-busy` and `instances` SHALL take `--from` / `--to` as inclusive days, as `event list` does, so one date spelling serves the whole CLI.

### Requirement: What the gcal family declines
`channels` / `watch` SHALL NOT be exposed: a push channel delivers to an HTTPS endpoint the caller must host, which a CLI has not. `calendars.transferOwnership` SHALL NOT be exposed: it is an irreversible administrative act on a Workspace domain. `calendars.clear` SHALL NOT be exposed: it empties the primary calendar with no undo, and `calendar delete` covers the secondary case. These are declined on purpose, and recorded as such rather than left as gaps.

### Requirement: The component families against item
The shared API SHALL offer two kinds of view over the same resources. `item` is the raw, unfiltered one: it lists, reads, writes and deletes any iCalendar object by id, leaving the bytes untouched. The component families (`event`, `todo`, `journal`) are the projected ones: each keeps its own component kind, renders the columns that kind is read by, and `event` additionally draws a cal(1)-style agenda. All SHALL share the `-k/--calendar` selector and the same item API; only the rendering and the filter differ.

### Requirement: One family per stored component kind
The shared API SHALL offer one command family per iCalendar component kind a calendar stores: `event` for VEVENT, `todo` for VTODO and `journal` for VJOURNAL. Each SHALL cover `list`, `read`, `create`, `update` and `delete` and read the same item API, so a component family costs a projection and a table and adds no backend operation. An item whose bytes do not parse SHALL project nothing rather than fail the listing.

### Requirement: Columns follow the kind
A component listing SHALL render the properties its kind is read by: an event its summary and its start and end, a todo its summary, due date, status, priority and completion percentage, a journal its summary and its date. A property the component omits SHALL render empty rather than absent, so the columns line up down the table. A todo whose STATUS is `COMPLETED` SHALL read as fully done even where PERCENT-COMPLETE is absent, since RFC 5545 3.8.1.8 makes the one imply the other.

### Requirement: Only events draw an agenda
`agenda` SHALL stay a VEVENT command. The grid marks the days that carry an event because an event occupies time; a todo's due date and a journal's date do not fill a day, and drawing them the same way would say they do.

### Requirement: A component window is applied locally
`event list` pushes its window down where the backend can narrow server-side. `todo list` and `journal list` SHALL apply theirs after parsing instead: a server-side range filter is defined against a component's start and end (RFC 4791 9.9), which a todo (due, no start) and a journal entry (dated, no end) do not both carry, so a pushed-down filter would drop them for the wrong reason. A component carrying no date at all SHALL show only in an unfiltered listing.

### Requirement: Kinds with no family
VFREEBUSY and VTIMEZONE SHALL NOT get a family. A VFREEBUSY is the answer to a query rather than a resource a calendar stores, and where a backend offers such a query it belongs to that backend's own protocol-specific family. A VTIMEZONE defines the zones the other components reference and is not an item a user lists.

### Requirement: Projections never rewrite bytes
Projecting a VEVENT out of an item SHALL be read-only and lossy by design: calendula returns what the backend holds. An item whose bytes do not parse SHALL project no event rather than fail the listing, so one malformed resource cannot hide a whole calendar.

"What the backend holds" is verbatim for the backends that store iCalendar, and synthesized for the one that does not; the [projection](./projection.md) capability governs the difference, and no command layer SHALL rewrite bytes of its own.

### Requirement: Time-range filtering
`event list` SHALL accept `--from` and `--to` as inclusive days. The pair SHALL map onto a range whose upper bound is exclusive (the day after, at midnight), so `--to` covers the whole day named. A crossed pair SHALL be rejected by name. A range SHALL lift the default page-size cap, so a window returns every match rather than its first page.

### Requirement: Calendar selection
A shared command operating inside a calendar SHALL take it through the flattened `-k/--calendar` flag, resolved by the account: the flag wins, otherwise `calendar.default`, otherwise the command bails. `calendar delete` is the one exception and SHALL inline a mandatory `-k/--calendar`, never falling back to a default.

### Requirement: Nested execute
Each subcommand SHALL be a clap-derived struct carrying its own arguments, with an `execute(self, printer, client)` method. `CalendulaCommand::execute` SHALL be the single dispatch point: it loads the configuration, selects the account, builds the appropriate client and hands it over.

### Requirement: Output goes to stdout
All data and errors SHALL go to stdout through the printer, with `--json` switching every command to JSON; stderr SHALL carry logs and prompts only. A command SHALL return a `Serialize + Display` value to the printer rather than printing inline.

### Requirement: Help is the usage reference
Each command's doc comment SHALL be its help text: the first paragraph is what `-h` shows, and the full text, ending with the command's JSON output shape, is what `--help` shows. `calendula <command> --help` is therefore the canonical usage reference for both humans and agents, which is why the README documents no per-command usage.
