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
`caldav`, `pimdir` and `vdir` SHALL each expose what only that backend has, gated behind its own cargo feature. CalDAV covers `discover`, its own calendar listing (carrying the ctag, the sync token and the accepted component kinds), `create` and `delete`. vdir covers its collection verbs including `rename`, which the shared API has no home for. pimdir covers `status`, reporting the source writes are attributed to, every source the store has been synced as, and how much of each calendar is downloaded.

### Requirement: event against item
The shared API SHALL offer two views over the same resources. `item` is the raw, unfiltered one: it lists, reads, writes and deletes any iCalendar object by id, leaving the bytes untouched. `event` is the VEVENT-focused one: its listing projects summary and time columns and drops every other component kind, and `event agenda` draws a cal(1)-style grid marking the days that carry one. Both SHALL share the `-k/--calendar` selector and the same item API; only the rendering and the filter differ.

### Requirement: Projections never rewrite bytes
Projecting a VEVENT out of an item SHALL be read-only and lossy by design: calendula stores what it was given and returns what was stored. An item whose bytes do not parse SHALL project no event rather than fail the listing, so one malformed resource cannot hide a whole calendar.

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
