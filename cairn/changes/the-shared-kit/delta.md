---
cairn: delta
change: the-shared-kit
---

# Delta

## ADDED Requirements

### Requirement: Text output is plain text
(commands) A text output SHALL be plain text: no border, no colour, no terminal escape, nothing sized to or truncated for the terminal, and it SHALL end with one newline; no command takes a width. The raw iCalendar a command is asked for (`event read`, `item read`, a `build` printing the item) SHALL print as it is stored. Every string a server, a store or a file gave (an id, a name, a summary, a description, an error) SHALL have its control and bidi characters replaced before it prints as text (pimalaya-cli `sanitize::text`); the JSON output SHALL carry it unchanged.

`event list` and `event find` SHALL print as Himalaya's `envelope list` does. Above the events, `Account:` (the account name, then the backend it resolved to in parentheses), `Calendar:` and, for `event find`, `UID:`, then a blank line. Then one line per event through pimalaya-cli's `Columns` with no header row: its item id, its start and its end (Columns follow the kind), then its summary, last, `-` for an empty cell. Under them, after a blank line when any printed, `Page:` (the page number, `all` for an unpaged listing, then the events the page holds, `event N` or `events N to M` followed by `, the last` when no page follows, or `empty`) and, while a page follows, `Next page:` and the `--page` reaching it, these values starting in one column, two spaces after `Next page:`. The events a footer counts are the calendar's events, the resources a page holds: a series counts once, however many lines its overrides or its occurrences in a window print. `, the last` and `Next page:` SHALL be exact on every backend: the unified client asks for one item past the page. `-s 0` lists every item, unpaged.

`item list` SHALL print the same header and footer around one line per item: its entity tag, its size, then its id. `gcal instances` SHALL print them around one line per instance: its id, its status, its start and end as the server spells them, then its summary, `Event:` naming the series in the header, the footer unpaged.

A list whose items carry more than one value of unbounded length SHALL print through pimalaya-cli's `Records`, the way `git log` prints commits: per item, its id alone on a line, followed by its state words in parentheses, comma-separated, when it has any; its properties as `Fields` prints them; its free text after a blank line, each line indented by four spaces; one blank line between items. A field every item carries SHALL print even when empty, its label alone; a field only some items carry SHALL be left out where it is empty. These lists print as records:

- `calendar list`: the calendar id; `default` for the one the server names so; `Name` and, when set, `Color`; the description.
- `caldav list`: the collection id; `Name` and, when set, `Color`, `Ctag`, `Sync token` and `Components`; the description.
- `gcal calendars`: the calendar id; `primary`; `Name` and, when set, `Role`, `Time zone`, `Color` and `Reminders`; the description.
- `gcal free-busy`: the calendar id; `busy` or `free`, neither when the answer failed; `Busy`, one `<start> to <end>` line per period, and `Errors`.
- `pimdir search`: the item id; `Start`, `End`, `Calendar` and, when a text term matched, `Matched`; the summary, then the snippet after a blank line. What the answer could not see prints after a blank line.

Every other list SHALL print through pimalaya-cli's `Columns`: a header of uppercase labels, then one item per line, the cells two spaces apart and padded to the widest of their column, an empty cell as `-`, the free-text column last, several values in one cell joined with `,`: `account list` (NAME, BACKENDS, DEFAULT), `vdir list` (ID, PATH, NAME), `gcal acl list` (ID, SCOPE, ROLE, VALUE), `gcal colors` (PALETTE, ID, BACKGROUND, FOREGROUND), and the calendars (ID, DOWNLOADED, QUEUED, NAME) and parked actions (ROW, ACTION, CALENDAR, ERROR) of `pimdir status`.

A record SHALL print through pimalaya-cli's `Fields`: one `Label: value` line per field, the values aligned past the longest label: `account check` (`Account:`, then one `<backend>:` line, `OK` or `FAIL (<why>)`), `caldav discover`, `gcal settings` (one field per setting) and the head of `pimdir status` (`Reading account:`, `Grouped accounts:`).

The `event agenda` grid SHALL stay cal's drawing, with no attribute: no day is highlighted, the events under it naming their days.

#### Scenario: An event list
- GIVEN the vdir account `perso` whose calendar `personal` holds the timed events `a`, `b` and `c`, read in UTC
- WHEN `calendula event list -s 2` prints them
- THEN it prints `Account:    perso (vdir)`, `Calendar:   personal` and a blank line, then `a  2026-11-12 09:00  2026-11-12 09:30  Standup` and the line of `b`, aligned, then a blank line, `Page:       1, events 1 to 2` and `Next page:  --page 2`, with no border and no colour
- AND `calendula event list -s 2 -p 2` prints the line of `c` and `Page:       2, event 3, the last`

#### Scenario: A calendar list
- GIVEN an account holding the calendars `personal`, described, and `work`
- WHEN `calendula calendar list` prints them
- THEN it prints `personal`, its `Name:` and `Color:` lines and its description indented by four spaces, a blank line, then `work` and its `Name:` line

### Requirement: Help is plain text
(commands) `--help` and `-h` SHALL print without colour, each paragraph on one line, never wrapped to the terminal width, the top-level help ending with the Pimalaya footer (where to report bugs, how to sponsor).

### Requirement: A query error is drawn without colour
(commands) The report `pimdir search` and `pimdir count` draw over a query they refuse SHALL carry no colour escape, on a terminal as in the `--json` error string.

### Requirement: A bare invocation shows the help
(commands) A bare `calendula` SHALL print the help when a configuration exists, a broken one included, its error left to the first command reading it. Without one it SHALL fail as a command loading the configuration does: the welcome, or the error. Nothing prompts.

### Requirement: A missing configuration meets a person with the welcome
(config) When no configuration exists at the first `-c` path or the default location, and stderr is a terminal with `--json` off, a bare `calendula` and every command loading the configuration SHALL print pimalaya-cli's welcome on stderr and exit 1, printing nothing on stdout. The welcome SHALL frame Calendula in one sentence, name the missing path, give the commands creating it from config.sample.toml (copied from share/doc/calendula next to the binary when one ships, downloaded with `curl` otherwise), invite the reader to edit it, and give the `event list` command carrying the `-c` paths.

Otherwise the command SHALL fail naming the path it looked at and the sample, as the failure object under `--json`. calendula SHALL NOT prompt to create a configuration.

#### Scenario: A mistyped path in a script
- GIVEN no file at `~/.config/calendula/confg.toml`, and stderr a pipe
- WHEN `calendula -c ~/.config/calendula/confg.toml calendar list` runs
- THEN it fails with `No configuration found at <that path>, start from the sample: https://github.com/pimalaya/calendula/blob/master/config.sample.toml`, and prints no welcome

#### Scenario: A first run at a terminal
- GIVEN an empty home and a terminal
- WHEN `calendula` runs
- THEN stderr says `Welcome to Calendula, the CLI to manage calendars.`, names `~/.config/calendula/config.toml`, gives the `curl` command creating it and then `calendula event list`, and the command exits 1

### Requirement: The configuration path comes from `-c` only
(config) `-c/--config` SHALL take one path per occurrence, never split on a delimiter, and MAY be repeated: the first path is the base and the next ones are deep-merged on top, in order, which is how a public configuration and a private one stay separate files. No environment variable SHALL be read for it: a path kept in one goes through the flag, `-c "$CALENDULA_CONFIG"`.

#### Scenario: A path holding a colon
- GIVEN `-c 'one.toml:two.toml;three.toml'`
- WHEN calendula reads its flags
- THEN it reads one path, spelled as given

### Requirement: Account resolution failures name what is missing
(config) The account SHALL be resolved through pimalaya-config's `TomlConfig::load` and `take_account`: the account `-a` names, else the one marked `default = true`. Several accounts marked default, layers and `-c` files merged, SHALL be refused, naming them sorted, never one taken at random; an unknown `-a` name SHALL be refused, listing the accounts the configuration holds; no default and no `-a` SHALL be refused, naming both ways of picking one. An account named with `-a` SHALL resolve whatever the defaults.

#### Scenario: Two defaults
- GIVEN a configuration marking `work` and `perso` with `default = true`
- WHEN `calendula event list` runs
- THEN it fails with `Several accounts are marked default (perso, work)`, and `calendula -a work event list` runs on `work`

### Requirement: No colour or table is configured
(config) The configuration SHALL hold no colour and no table shape. A `table` table (`table.preset`, `table.arrangement`), at the top level or under an account, and a top-level `account` table (`account.list.table`) SHALL fail the load by name, saying the output is plain text and to remove it. A `*-color` key under `event.list.table` or `item.list.table`, or a `calendar.list` block, SHALL fail the load as an unknown field of the block that holds it. `event.list.page-size` and `item.list.page-size` stay.

#### Scenario: A colour key
- GIVEN a configuration written for 0.2 with `event.list.table.id-color = "red"`
- WHEN any calendula command loads it
- THEN it fails naming the line and the unknown field `table`, where `page-size` was expected

## MODIFIED Requirements

### Requirement: Columns follow the kind
(commands) An event listing SHALL show each event's summary and its start and end.

The start and end SHALL show for a person to read, `startsAt` and `endsAt` in the local zone (`TZ`, then the system's), the one a floating time is read in: `2026-10-30 09:00` for a timed event, the days for a whole-day one with the end as its last day (`2026-05-19`), never before its first, so a zero-length one (`DTEND` equal to `DTSTART`) ends on its start. A time that did not resolve SHALL show as iCalendar spells it. Nothing is truncated. The `--json` payload SHALL stay as it is, `start` and `end` iCalendar-spelled and a whole day's `endsAt` exclusive.

A property the event omits SHALL print as `-` rather than nothing, so the columns line up down the list.

### Requirement: Only events draw an agenda
(commands) `agenda` SHALL stay a VEVENT command: the events listed under its grid, by the day and time they start, are VEVENTs, an event being what occupies time.

### Requirement: An agenda expands recurring series
(commands) The agenda SHALL read the events over the days its grid draws and SHALL expand a recurring series over them, as a windowed `event list` does: each occurrence lists under the grid at its own day and time, overrides applied and excluded dates dropped.

A series placed once at its first start leaves a yearly event out of every year but its first, so the agenda and a listing of the same days would disagree.

#### Scenario: A yearly event created years ago
- GIVEN a calendar holding a yearly event whose first start is in 2019
- WHEN `calendula event agenda 2027` draws that year
- THEN the event lists under the grid at the day it falls on in 2027

### Requirement: An agenda reads its date as cal does
(commands) The agenda SHALL read one argument that is a number as a year and drawn whole, and one argument that is not a number as a month name, drawing that month of the current year. A day that is not a number of the month, a month that matches no name in English or in the time locale, a month number outside 1-12, or a year that is not an integer SHALL be refused, naming the value, never replaced by the 1st, by January or by the current year.

The grid is a port of cal, and a typo silently drawing another month reads as an empty one.

#### Scenario: A lone month name
- GIVEN `LANG=es_ES.UTF-8` in 2026
- WHEN `calendula event agenda noviembre` runs
- THEN it draws November 2026 alone, as `cal noviembre` does

#### Scenario: An unknown month name
- GIVEN `LANG=es_ES.UTF-8`
- WHEN `calendula event agenda novembre 2026` runs
- THEN it fails with `Unknown month name: novembre` and draws nothing

### Requirement: Nested execute
(commands) Each subcommand SHALL be a clap-derived struct carrying its own arguments, with an `execute(self, printer, client)` method.

`Command::execute` SHALL be the single dispatch point: it loads the configuration, selects the account, builds the appropriate client and hands it over.

### Requirement: Output goes to stdout
(commands) All data and errors SHALL go to stdout through the printer, with `--json` switching every command to JSON; stderr SHALL carry the logs, the prompts and the welcome only. A command SHALL return a `Serialize + Display` value to the printer rather than printing inline.

### Requirement: pimdir searches calendar items through the store's index
(commands) `calendula pimdir search <query>...`, gated on the `pimdir` feature, SHALL answer the pimdir SEARCH query language (§8) from the store's search index, scoped to the account's events: the query is read as its conjuncts plus `kind:event`, `account:<account>` when the store groups the account under a name, and `collection:<id>` when `-k/--calendar` names a calendar, each a top-level conjunct. Without `-k` it SHALL search every calendar of the account, `calendar.default` not applied; a `-k` naming no calendar of the account SHALL be refused, naming those it holds. The words of the query are joined by spaces into one query. Calendula SHALL open the index read-only (io-pimdir `PimdirIndex::open_query`) and SHALL NOT create, rebuild nor refresh it: the store's owner, its sync engine, does. The answer SHALL be the store as of the owner's last refresh: a queued write is not listed nor folded in.

A `date:` term SHALL compare the occurrences the index expanded (SEARCH §7), its days, `today` and `yesterday` read in the zone `event list` reads times in (`TZ`, then the system's), stated to the index as an IANA name (SEARCH §8.1); a zone with no IANA name SHALL read them in UTC, with a warning.

Each hit SHALL print as a record (Text output is plain text): its id, the public `seq`, which `event read` takes; its start and end as `event list` shows them (Columns follow the kind), from its summary when its body is not local; its calendar; what a text term matched (`body` or `summary`) when one did; then its summary and its snippet. `--sort date|name|relevance` orders the hits, `date` (latest first) by default, a series by its first occurrence, and `--page` and `--page-size` page them, the page size defaulting to `event list`'s. Under `--json` the page carries `hits`, `more` (whether hits follow it) and `coverage`, each hit the event projection plus `calendarId`, `matched` (`null` when only field terms matched) and `snippet`.

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

### Requirement: A pimdir calendar search says what it did not cover
(commands) The output of `pimdir search` and `pimdir count` SHALL carry the coverage of the answer (SEARCH §8.2): `bodiless`, the items in scope whose body is not local, matched on their summary alone; `unindexed`, those whose last change the index has not folded in; `behind`, how many changes the store drew above the index, store-wide; `since` and `until`, the range below or above which the store is not known to hold every item; `filling`, whether a first round is still listing a calendar in scope; `horizonLeft`, whether the date range reaches outside the occurrences the index holds; `cut`, how many series stop their occurrences before the range's end; and `complete`, true only when none of these holds. The plain output SHALL print one line after its hits, a blank line apart, naming what did not hold, and nothing when the answer is complete.

#### Scenario: A range past the horizon
- GIVEN an index holding occurrences two years forward
- WHEN `calendula pimdir search date:2030` runs
- THEN the coverage reports `horizonLeft` true and `complete` false

### Requirement: A pimdir search names an index read in another zone
(commands) When the index reads floating times and dates in another zone than the one `pimdir search` and `pimdir count` read days in (SEARCH §7, `floating_zone`), their plain output SHALL say so after the hits or the count, naming both zones; the JSON output SHALL stay unchanged.

#### Scenario: An index read in another zone
- GIVEN `TZ=Europe/Paris` and an index reading floating times in `UTC`
- WHEN `calendula pimdir search date:2026-11-07` runs
- THEN the plain output says the index reads floating times and dates in UTC and this search reads days in Europe/Paris

### Requirement: Loading and merging
(config) Without `-c`, a configuration SHALL be read from every existing canonical path, each a layer deep-merged on top of the ones before: `$XDG_CONFIG_HOME/calendula/config.toml`, `$HOME/.config/calendula/config.toml`, then `$HOME/.calendularc`, a path naming a file already loaded skipped. `-c` SHALL replace them (The configuration path comes from `-c` only).

### Requirement: Unknown fields
(config) `deny_unknown_fields` SHALL be set on the leaf blocks, so a typo in an option is reported rather than ignored, and SHALL NOT be set on the top-level and account blocks, whose backend blocks exist only in the builds carrying their feature: a build without a backend still loads a file naming it. A table calendula no longer reads SHALL be refused there by name instead: `todo`, `journal`, `table` and the top-level `account`.

### Requirement: Backend blocks
(config) An account SHALL carry an optional block per compiled backend: `vdir` with a `home-dir`, `pimdir` with a `root` and an optional `account`, `caldav` with its endpoint, TLS, ALPN and authentication, `gcal` and `msgraph` with TLS, ALPN and authentication.

An account MAY carry several; which one a unified command uses is the backend capability's business.

Every paged listing family SHALL additionally carry a block naming its default page size: `event.list` and `item.list`.

### Requirement: A credential command runs once per account
(config) Every secret an account resolves in one run SHALL go through a single resolver, so a command two of its backend blocks name is spawned once and its value handed to both. A `pass` or `gpg` entry shared by `caldav` and `gcal` therefore costs one unlock per account check, not one per backend.

Distinctness is the configured command's own shape, and never crosses the two of them: a shell line and the argv spelling that runs it through the platform shell are two commands.

The resolver holds plaintext for as long as it lives, so it SHALL be built where the backends of one account are assembled, the unified client of the account or `account check`, and dropped with them, never held for the life of the process. The unified client SHALL keep it across the reconnection that follows a composer, so a reopened backend runs no secret command again.

A caller opening a single backend SHALL keep resolving through a resolver of its own, so the shape stays the same everywhere and nothing outlives its account.

### Requirement: A backend names the ALPN it offers
(config) A backend whose protocol pins an HTTP version SHALL expose an `alpn` key, a list of identifiers offered during the TLS handshake. Unset offers the protocol default, an explicit empty list skips ALPN negotiation and a non-empty list replaces the default.

`caldav.alpn`, `gcal.alpn` and `msgraph.alpn` default to `http/1.1`, the only version io-http speaks.

The runtime TLS profile SHALL be built through `TlsConfig::into_tls(alpn)`, never through a conversion assuming a list, so no caller can negotiate a protocol it did not name.

### Requirement: Released dependencies, patched only while unreleased
(packaging) Dependencies SHALL name released versions. A `[patch.crates-io]` entry is allowed only while a needed change rides unreleased upstream, the cairn change adding it SHALL say what it waits on (Cargo.toml carrying no comment in its dependency or patch sections), and it SHALL be dropped as that crate publishes.

## REMOVED Requirements

### Requirement: A missing configuration is an error
(config) Replaced by A missing configuration meets a person with the welcome.

### Requirement: Table rendering keeps its preset string
(config) The table and its preset are gone (No colour or table is configured).
