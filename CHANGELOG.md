# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Calendula 1 is the first stable release, aligned on the other Pimalaya CLIs. It adds the JMAP backend, the `msgraph` namespace, the raw CalDAV methods, `event search`, the pimdir queue and scheduling commands, a REPL and an MCP server, and every `--json` failure carries a stable `code`. The configuration wizard, the tables and colours, and the `todo` and `journal` commands are gone. A 0.x configuration is not carried over: start over from config.sample.toml.

A pimdir store an earlier neverest wrote is refused `store-version`, never migrated: let that neverest push its queue, move the store aside and sync a new one.

### Added

- Added the `jmap` backend, in the default features, serving the unified API over JMAP for Calendars.

  `auto` takes it after CalDAV and before gcal. A session not advertising `urn:ietf:params:jmap:calendars` is refused when the backend opens, and a `todo` or `journal` listing answers empty.

  A write the server refuses for its `iCalendar` member is sent again without it, with a warning naming what was dropped.

- Added the `jmap` account block (`server`, `tls`, `alpn`, `proxy`, `auth` as `basic`, `bearer` or `header`), a bare server name discovered through `/.well-known/jmap`.
- Added the `msgraph` namespace: `calendars` (`list`, `get`, `create`, `rename`, `delete`), `events` (`list`, `get`, `create`, `update`, `delete`, `delta`), `profile get` and `request`.

  Each prints Graph's own JSON. The listings print `nextPage`, taken back by `--page-token`, and `--top` sizes a page. `request` refuses a URL off Graph's origin before sending, coded `request-origin`.

- Added the WebDAV and CalDAV methods to `caldav`: `propfind`, `proppatch`, `mkcalendar`, `rmcol`, `report` (`query`, `multiget`, `sync`, `raw`), `get` and `put`.

  `caldav list` and `caldav create` stay hidden aliases of `propfind` and `mkcalendar`. `caldav discover` adds `server` and prints an endpoint it cannot resolve as `null`.

- Added `gcal request <METHOD> <PATH> [JSON]`, sending any Calendar API request and printing `{response}`; a URL off Google's API origin is refused before sending, coded `request-origin`.
- Added `vdir item` (`list`, `get`, `add`, `update`, `delete`, `copy`, `move`), the raw files of a collection of any kind.
- Added `event search <QUERY>...`, searching the account's events in the pimdir query language.

  Only pimdir serves it (capability `event.search`), the other backends refusing it `capability-missing`. It prints `{events, nextPage, complete, errors}`, and `--sort` takes `newest-first` (the default), `oldest-first`, `title` or `relevance`.

- Added `pimdir search <QUERY>...` and `pimdir count <QUERY>...`, querying the store's search index over the account's events, every calendar unless `-k` names one.

  `--json` prints `{hits, nextPage, coverage, errors}`, each hit the event with its calendar, what matched and a snippet, `coverage` saying what the index could not see. `--sort` reads as on `event search`, and `--page`, `--page-size` and `--page-token` page it.

  The query takes `date:` and `when:` (days in the local zone, instants, `min`, `h`, `d`, `w`, `mo` and `y` relative units), `label:` and `property:<key>=<value>`. A reserved form is refused `query-refused` unless quoted, and a term starting with `-` goes after `--`.

- Added `pimdir queue list`, `pimdir queue show <ROW>` and `pimdir queue cancel <ROW>`, reading and retracting the writes queued in the store.

  `queue list --json` prints `{actions, parked, errors}`, each row `{row, handle, kind, collection, linkId, itemId, producer, queuedAt, attempts, unconfirmed, error, summary}`. A row that does not decode is an `errors` entry coded `action-malformed`.

  `queue show` prints that row plus `state` (`pending`, `parked`, `unconfirmed`, `applied`), `appliedAt` and `refusals`. `show` and `cancel` take a row id or a handle (`<store id>.<incarnation>.<row>`), failing `row-not-found` on an unknown row and `store-reset` on another store's handle.

  `queue cancel` asks unless `-y`, failing `confirmation-required` with no terminal, and fails `store-owned` while a sync runs.

- Added `pimdir reply` and `pimdir cancel`, queueing a reply to an invitation or the cancellation of an event the account organizes, sent by the next sync.

  `--source` names the source sending it, needed when several can, and `--recurrence-id` limits it to one occurrence. Each prints `{row, handle, id, calendarId, recurrenceId, partstat, comment, source}`.

  An event not downloaded fails `body-pending`, one naming no `ATTENDEE` `invitation-missing`, and one no source can act on `no-performer` or `ambiguous-performer`.

- Added `parked` and `refused` to `pimdir status`, the writes the sync engine could not apply and the events a server refused for good.

  `--json` prints each parked row `{row, handle, collection, kind, error}` and each refused event `{calendarId, itemId, source, reason, refusedAt}`.

- Added `repl`, a session running the CLI's commands over one account, connecting once.

  A line takes `--json` and a `#` comment, and `-V` prints the version. A nested `repl` or `mcp` and the flags the session fixes (`-a`, `-c`, `-b`, `--log-level`) are refused `usage-invalid`, the composer and stdin input `stdin-taken`, the session going on.

- Added `mcp`, a Model Context Protocol server over stdio exposing the unified API as tools.

  It lists `account_capability`, `calendar_list`, `event_list`, `event_read`, `event_search`, `item_list` and `item_read`. `--allow-write` adds the calendar, event and item add, update and delete tools, and `--allow-send`, which needs it, the `attendee`, `add-attendee`, `remove-attendee`, `organizer` and `send-updates` arguments.

  Without `--allow-send`, a write or a delete touching an item that names an `ATTENDEE` is refused `sending-closed`. A tool the backend does not serve is not listed, and no tool reads a local path.

- Added `account capability`, listing what an account serves of the unified API without connecting, `{account, capabilities: [{name, backend, level, detail}]}` under `--json`.

  `--backend <name>` reads another backend of the account. Each unified command's `--help` ends with a `Backends:` footer naming the backends serving it.

- Added `accounts`, a hidden alias of `account`.
- Added the `backend-not-built` failure: a command reaching a backend the build leaves out names the cargo feature bringing it back (`--features gcal`).

  The backend's block still loads, unread, so one configuration serves every build. `account list --json` names such backends in `notBuilt`, and `account check` fails their row.

- Added a stable `code` to the `--json` failure object, to each `errors` entry and to each `account check` row, `null` when none applies.

  Usage and accounts: `usage-invalid`, `stdin-taken`, `capability-missing`, `backend-not-built`, `account-missing`, `account-default-missing`, `account-default-several`, `calendar-default-missing`, `request-origin`, `sending-closed`, `confirmation-required`, `page-token-invalid`, `row-not-found`.

  Items and calendars: `item-not-found`, `calendar-not-found`, `item-duplicate`, `calendar-duplicate`, `item-invalid`, `item-unreadable`, `body-pending`, `precondition-failed`, `occurrence-unsupported`, `invitation-missing`. A missing item fails with one message whatever the backend.

  Server refusals: `permission-denied`, `quota-exceeded` and `rate-limited`, the first 429 met, calendula retrying nothing.

  pimdir store: `store-uncreated`, `store-version`, `store-foreign`, `store-corrupt`, `store-not-current`, `store-hash-algo`, `sync-version`, `store-account-ambiguous`, `store-account-unknown`, `lock-replaced`.

  pimdir writes: `store-busy`, `store-owned`, `store-staging`, `store-in-flight`, `store-exposed`, `store-reset`, `flag-invalid`, `action-malformed`, `conflict-unread`, `unsupported`, `no-performer`, `ambiguous-performer`.

  Search index: `index-missing`, `index-outdated`, `index-version`, `index-zone-unknown`, `index-fts5-missing`, `query-refused`, `query-unanswerable`.

- Added a schema for every command to `json-schema`, with `calendula-error` for the failure object and `calendula-item-error` for an `errors` entry.
- Added `errors` to `calendar list`, `event list`, `event search`, `event agenda` and `item list`: one `{id, code, error, sources}` per calendar or item the listing could not read.

  `id` is `null` when the backend named none (a CalDAV listing the server truncated). The text prints one `Error:` line each, after the output.

- Added `nextPage` to `event list --json` and `item list --json`, and `--page-token` taking it back, winning over `-p`.
- Added `calendarId`, `etag`, `uid`, `recurrenceId`, `location`, `allDay`, `timeZone`, `zoneAssumed`, `recurring`, `sequence`, `status`, `transparency`, `busyStatus`, `organizer`, `attendees`, `onlineMeetingUrl` and `alarms` to the event JSON.

  `onlineMeetingUrl` prefers a video `CONFERENCE`, then a web one, never a dial-in. `zoneAssumed` tells a `TZID` nothing defines, read in the local zone. On pimdir, `etag` is the store's hash of the body.

- Added recurrence expansion to `event list --from/--to`: each occurrence overlapping the window lists on its own with its `recurrenceId`, overrides applied and excluded dates dropped.
- Added `event read --recurrence-id`, projecting one occurrence.
- Added `event read --uid <UID>`, reading the event carrying that iCalendar `UID`.
- Added `--if-match` to `event delete` and `item delete`, mapped to each backend's version check; vdir refuses it `capability-missing`.
- Added field flags to `event build`, `event add` and `event update`, stacked between the source and `-i`.

  The flags are `--uid`, `--summary`, `--description`, `--location`, `--url`, `--start`, `--end`, `--duration`, `--time-zone`, `--status`, `--transparency`, `--organizer`, `--attendee`, `--categories` and `--sequence`, plus `--method` on `event build`.

  A flag replaces every instance of its property and leaves the other lines as they were; an empty value removes it. A time takes a date, a local time, a `Z` time or an IANA zone prefix (`Europe/Paris:2026-10-19T09:00`), a zone named bringing its `VTIMEZONE`.

  `event update --start` alone moves the end with it, a time naming no zone being read in the zone of the event's `DTSTART`. On Google and Outlook, a new event's time naming no zone is read in the local zone unless `--time-zone` names one.

- Added `--add-attendee [NAME:]ADDRESS` and `--remove-attendee ADDRESS` to `event build`, `event add` and `event update`, each repeatable, every other line kept as it was ([#22](https://github.com/pimalaya/calendula/issues/22)).

  Addresses compare case-insensitively, `mailto:` or not. A redundant or conflicting use, or either flag beside `--attendee`, is refused before anything is written.

- Added `--alarm <LEAD>` and `--no-alarm` to `event build`, `event add` and `event update`, replacing every VALARM with one display alarm the lead before the start, or removing them all.

  The lead is an RFC 5545 duration (`PT15M`) or `15m`, `1h`, `2d`. A lead the backend cannot hold is refused, never rounded.

- Added `--online-meeting` to `event build`, `event add` and `event update`, asking the sync engine for a Google Meet or Microsoft Teams meeting (`X-PIMDIR-ONLINE-MEETING:TRUE`).

  Only pimdir serves it, refusing it when no source pushing the event declares `calendar.online-meeting`.

- Added `--send-updates all|external-only|none` to `event add`, `event update` and `event delete` on gcal accounts, choosing whom Google mails ([#21](https://github.com/pimalaya/calendula/issues/21)).

  Left out, nothing changes. `all` and `external-only` are refused on an event organized by someone else, which is imported, and other backends refuse the flag `capability-missing`.

- Added the event projection to `event build --json`, `{contents, encoding, method, events}`, so a loose iCalendar reads with no account; its events print `id` and `calendarId` `null`.
- Added `default` to each calendar of `calendar list --json`: the server's default calendar, Google's primary, Graph's `isDefaultCalendar` or the role pimdir recorded, `false` on CalDAV and vdir.
- Added `description` and `color` to `vdir list --json`, and a COLOR column to its text.
- Added the account-level `tls` table, each network backend's `tls` taking from it, field by field, what it leaves unset.
- Added `tls.client.cert` and `tls.client.key`, a PEM client certificate and its key presented when the server asks for one.
- Added the account-level `proxy` table (`url`, `username`, `password`), a SOCKS5 or HTTP proxy for every network backend, overridden per backend under the same keys.

  Without one, `all_proxy` and `https_proxy` are read, honouring `no_proxy`.

- Added `caldav.alpn`, the ALPN identifiers offered during the TLS handshake, `["http/1.1"]` when unset, an empty list skipping ALPN.
- Added the first-run welcome: a command run at a terminal without a configuration prints on stderr how to create one from the shipped sample, then exits 1.

  Under `--json` or with stderr piped, the error names the path looked at and the sample.

- Added config.sample.toml to the release archives and the Nix package as share/doc/calendula/config.sample.toml, which install.sh copies under its prefix.

### Changed

- **BREAKING**: Changed the pimdir backend to the current pimdir drafts, refusing an earlier store `store-version` and an earlier search index `index-version`, never migrated.

  The sync engine rebuilds the index.

- **BREAKING**: Changed the pimdir backend to read one account of the store: with `pimdir.account` unset, the one account holding calendars, several being refused `store-account-ambiguous`.

  A `pimdir.account` naming no account of the store fails `store-account-unknown`, and a collection declaring no kind is no calendar.

- **BREAKING**: Changed a pimdir item whose body is gone from the store to read as not downloaded: listed with no bytes and `etag` `null`, its reads failing `body-pending`.
- **BREAKING**: Changed a pimdir `item add` or `item update` given an iCalendar (a file, contents or stdin) to queue it without its `X-PIMDIR-` properties.
- Changed the pimdir writes to be refused before they are queued when a source of the store does not support them, coded `unsupported`, naming the capability and the source.

  A to-do into a calendar taking none is refused so. A scheduled event refused for want of a source that notifies names the way through: `SCHEDULE-AGENT=NONE` on its `ORGANIZER` and `ATTENDEE`.

- Changed a pimdir refusal to print its context as the error and the store's reason in `sources`.
- **BREAKING**: Changed every write printing a confirmation to print a receipt under `--json`, where it printed `{"message": ...}`.

  `calendar update|delete` and `event|item add|update|delete` print `{id, queued}`, `queued` telling a write waiting for a sync, and `calendar create` `{id, name, queued}`. The `caldav`, `vdir` and `gcal` writes print `{id}`, and `event|item build -o` `{path, size}`.

  On pimdir, the text says the write is queued for the next sync, and a queued create's `id` is the link id it staged. `build -o` prints `Saved <size> bytes to <path>`.

- **BREAKING**: Changed the exit codes to pimalaya-cli's contract: 1 for a failure, 2 for a command line clap refuses, 3 for a result that is not a full success.

  Under `--json`, a refused command line, and a bare `calendula --json`, print the failure object coded `usage-invalid` on stdout. Exit 3 covers a failed `account check` (0 before) and a listing whose `errors` is not empty.

- **BREAKING**: Changed a server's refusal of a `caldav`, `gcal` or `msgraph` command to be its result, `{refusal: {status, reason, message, code, retryAfter}}`, exit 3, where it failed with exit 1.
- **BREAKING**: Changed `account check --json` to `{accounts: [{account, backends}]}`, each backend `{backend, ok, code, error, tls}`.

  `tls` is the TLS configured, `null` for a local backend or a cleartext connection. On pimdir the check probes a producer, failing `store-busy` while a collector holds the staging lock, and writes nothing.

- **BREAKING**: Changed the unified commands to refuse a verb or a flag the account's backend does not serve with `capability-missing`, before resolving a credential, opening a store or connecting.

  The unified API is the union of what the backends serve; the message names the capability and the backend of the account serving it. `--if-match` on a vdir update, which was ignored, is refused so.

- **BREAKING**: Changed `auto` to stop on a backend the build leaves out, failing `backend-not-built`, where it took the next backend configured.
- **BREAKING**: Changed a listing to report an item it could not read or parse as an `errors` entry, coded `item-unreadable` when no other code applies, and list the others.

  vdir and Graph failed the whole listing, and an event that does not parse was skipped without a word.

- **BREAKING**: Changed the text output to plain text, with no border, no colour and nothing sized to the terminal.

  A listing prints one line per row under a header naming the account, the calendar and the page, then `Next page:`; the rest prints records or `Label: value` fields. Every string a server or a file gave is defused of control and bidi characters.

  `event agenda` writes no terminal escape, so it highlights neither today nor the days holding an event. `account list` prints one line per account with no header, `default` marking the default one.

- **BREAKING**: Changed `event agenda --json` to `{events, errors}`, each event `{id, start, label}` in the text's order, where it mapped local start times to labels.
- **BREAKING**: Changed `item list`, `item read` and `event read` to print an item as `{id, calendarId, etag, contents, encoding}`.

  `contents` is the iCalendar text, `encoding` `utf-8`, or the base64 of bytes that are not UTF-8, `encoding` `base64`, both `null` in `item list` for an item with no local bytes. `item list` printed `contents` as integers.

- Changed `item build --json` to print `{contents, encoding}` and `event build --json` to add `encoding`, both handing over bytes that are not UTF-8 in base64, where they failed.
- **BREAKING**: Changed an event's `summary`, `description` and `location` to print `null` when absent, where they printed `""`.
- **BREAKING**: Changed an event's `start` and `end` under `--json` to RFC 3339 with their offset (`Z` for UTC), or `YYYY-MM-DD` for a whole day with an exclusive end, where they were spelled as in iCalendar.
- **BREAKING**: Changed `gcal calendars`, `gcal acl list`, `gcal events instances`, `gcal settings`, `gcal colors` and `gcal free-busy` to print under `--json` what Google answered, every key kept.

  They printed rows calendula shaped. The text is unchanged.

- Changed `gcal instances`, `gcal move` and `gcal quick-add` to `gcal events instances`, `gcal events move` and `gcal events quick-add`, the flat names staying hidden commands.
- Changed `gcal free-busy` to require `--from` and `--to`.
- Changed `gcal acl update` to read the scope and its value from the rule id, `--scope` and `--value` now optional, one disagreeing with the id refused `usage-invalid`.
- Changed `event create` and `item create` to `event add` and `item add`, `create` staying a hidden alias.
- **BREAKING**: Changed four JSON schema names after their commands' new names.

  `calendula-caldav-list` and `calendula-caldav-create` are `calendula-caldav-propfind` and `calendula-caldav-mkcalendar`; `calendula-event-create` and `calendula-item-create` are `calendula-event-add` and `calendula-item-add`.

- Changed each `json-schema` schema to carry an `$id` naming the contract version.
- Changed piped `--json` output to print one value per line.
- **BREAKING**: Changed `caldav delete <CALENDAR> <ITEM-ID>` to delete one item, `caldav rmcol` deleting a calendar.
- **BREAKING**: Changed `calendar create` to take the name alone, `calendar create <NAME> [--id <ID>]`, the old `<ID> <NAME>` form exiting 2.

  Without `--id`, vdir names the directory after the name and CalDAV mints a UUID. Google and Graph, which mint the id, refuse `--id` `capability-missing`.

- Changed `calendar update` and `calendar delete` to take the calendar as an argument, `-k/--calendar` staying a hidden alias.
- **BREAKING**: Changed `-d` on `caldav mkcalendar` and `vdir create` to set the description, where it set the display name, now `-n`.
- Changed `--color` to take `-C` on `calendar create`, `calendar update`, `caldav mkcalendar` and `vdir create`.
- **BREAKING**: Changed `calendar create` and `vdir create` naming an existing calendar to fail `calendar-duplicate`, writing nothing, where vdir wrote over its marker files and exited 0.
- **BREAKING**: Changed a Google `event add` or `item add` naming a `UID` the calendar holds to fail `item-duplicate`, where it overwrote that event.

  A CalDAV create answered 412 fails `item-duplicate` too. Graph and vdir still take a second item of one `UID`.

- **BREAKING**: Changed a vdir `item update` or `event update` naming an id the calendar does not hold to fail `item-not-found`, where it created the item.
- **BREAKING**: Changed a CalDAV calendar or item id to print percent-decoded when it encodes back to the served segment (`My Cal`, not `My%20Cal`); an id printed before still addresses its resource.
- Changed a CalDAV item read answered 400 to fail `item-not-found`, as a 404 does.
- Changed `calendar update` on CalDAV to write the colour first in its own request, so a server ignoring it fails the update before anything else changes; `calendar create --color` warns when the colour was not kept.
- **BREAKING**: Changed `--from` and `--to` to read local days (`TZ`, then the system's zone), where they read UTC days, in `event list`, `caldav report query`, `gcal free-busy` and `gcal events instances`.
- Changed `-s 0` on `event list` and `item list` to list every item, where it listed none.
- Changed the local date windows (vdir, msgraph, pimdir) to keep a recurring event whose occurrences fall in them, not only one starting there.
- Changed `event agenda` to name months and weekdays in the time locale (`LC_ALL`, `LC_TIME`, then `LANG`), and to read a month argument in English or in that language.
- Changed `event update` to set `DTSTAMP`, and `LAST-MODIFIED` when present, to the time of the write.
- Changed the composed-item check to refuse a duration outside RFC 5545 3.3.6 and a `LANGUAGE` or `ALTREP` on a property the RFC gives none.
- Changed the gcal and msgraph backends to write an online meeting's join link as a `CONFERENCE` (RFC 7986) rather than their vendor property.
- Changed a Google account to accept an event ending by `DURATION`, stored with a `DTEND`.
- Changed a Google event holding more lines Google does not model than fit in 30,000 octets to be written without the ones that do not fit, with a warning.
- Changed an Outlook recurring event whose `RRULE` Graph cannot express exactly to be refused, where it was written as another series.
- Changed the Outlook calendar listing to refuse a paging link leaving the Graph origin.
- Changed CalDAV to address a calendar outside the home set by its path (Google's `/caldav/v2/<calendar>/events/`), not its last segment.
- Changed CalDAV to resend once a request the server closed the connection on.
- Changed a CalDAV listing answered with anything but one whole multistatus to fail, where it read as a shorter or empty listing, and a delete answered by a multistatus naming failed members to fail.
- Changed an HTTP response whose head passes 64 KiB or whose body passes 256 MiB to fail.
- Changed an iCalendar body to nest 16 components at most.
- **BREAKING**: Changed `-c` to take one path per flag, repeated to merge several files, never split on `:`; `CALENDULA_CONFIG` is no longer read.
- **BREAKING**: Changed the default configuration paths to load as layers: every existing one among $XDG_CONFIG_HOME/calendula/config.toml, $HOME/.config/calendula/config.toml and $HOME/.calendularc merges over the ones before.
- **BREAKING**: Changed the configuration to refuse a key it does not know, at the top level and in an account, naming it.
- **BREAKING**: Changed account resolution to refuse several accounts marked `default = true`, where one was taken at random; an unknown `-a <NAME>` lists the accounts.
- **BREAKING**: Changed `-a default` to take the account named `default`, not the one marked `default = true`.
- **BREAKING**: Changed `vdir.home-dir` to `vdir.root`.
- **BREAKING**: Changed a CalDAV `server` or `home` to take an ASCII host only, an internationalised one written in Punycode (`xn--...`), the refusal naming the host.
- Changed `caldav.auth` to be optional: a block naming none sends no `Authorization` header, as `auth = "none"` does.
- Changed config.sample.toml to Himalaya's shape: one commented key per line, alternatives in brackets, `proxy` and `tls` as flat keys written once, one section per backend (CalDAV, JMAP, Google Calendar, Microsoft Graph, vdir, pimdir), the provider sections moved to the README.

- Changed the clap help to plain, unwrapped text with the Pimalaya footer.
- Changed the `-b/--backend` help to list `auto` and every backend, and an unknown `-b` to be refused listing the possible values.
- Changed `--log-file` to refuse a symbolic link to an existing file.

### Removed

- **BREAKING**: Removed `-w/--max-width` from `calendar list`, `event list` and `item list`.
- **BREAKING**: Removed the table and colour options: `table.preset`, `table.arrangement` and the `*-color` keys of the `*.list.table` tables, the `page-size` keys staying.
- **BREAKING**: Removed the configuration wizard: the `configure` command (alias `wizard`), the `wizard` cargo feature and the `calendula-configure` schema.

  A bare `calendula` prints the help.

- **BREAKING**: Removed the `todo` and `journal` commands and their `todo.list` and `journal.list` options, to-dos moving to [unfog](https://github.com/pimalaya/unfog) and journal entries to [journal](https://github.com/pimalaya/journal).

  `item` still reads and writes any component raw.

- **BREAKING**: Removed `caldav.discover`, the RFC 6764 lookup of a bare domain: set `caldav.server` to the server's CalDAV root.

  A `caldav.server` naming an `https` origin is probed at `.well-known/caldav`, a redirect followed over `https` alone.

### Fixed

- Fixed a pimdir create or update losing its body when a collector ran before it was queued.
- Fixed a pimdir create whose `UID` the calendar holds or has queued being parked by the sync engine: it fails `item-duplicate` before anything is queued.
- Fixed a pimdir update ignoring `--if-match` and overwriting a change a sync landed while it waited in the queue: it is gated on the version it read or was given, and parked when that version moved.
- Fixed a pimdir write to a calendar no source syncs being queued for nobody: it fails `no-performer`.
- Fixed a pimdir event not downloaded yet vanishing from `event list` and `event agenda` while the page counted it.
- Fixed one pimdir queue row that does not decode failing every `event list` and `event agenda`: they read on, with a warning.
- Fixed a windowed listing aborting on a not-downloaded pimdir event whose stored `DTSTART` holds a multi-byte character.
- Fixed a windowed `event list -s` sorting inside each page of the backend's order: it pages its occurrences sorted across the window.
- Fixed the Graph backend ignoring the calendar named on an item read, update or delete, so `event delete -k A <id of B>` deleted from B: it fails `item-not-found`.
- Fixed `event update` and `item update -i` on Microsoft Graph failing `precondition-failed` right after a create, when Exchange restamped the event by itself.
- Fixed a msgraph listing failing on a `numbered` recurring series ([#10](https://github.com/pimalaya/calendula/issues/10)).
- Fixed msgraph listings dropping the cancelled occurrences of a series ([#11](https://github.com/pimalaya/calendula/issues/11)).
- Fixed msgraph listings repeating an event Graph returned on both sides of a page boundary ([#12](https://github.com/pimalaya/calendula/issues/12)).
- Fixed a msgraph listing failing whole on a recurring series with no end or bounded over five years ([#31](https://github.com/pimalaya/calendula/issues/31)).
- Fixed a msgraph `event list` over a range reading every series the calendar ever held, ended ones included ([#32](https://github.com/pimalaya/calendula/issues/32)).

  A series is read within the range, or five years either side of today without one; a window Graph refuses makes that series an `errors` entry.

- Fixed a Google or Outlook series losing its cancelled and moved occurrences on a write: each is written through its instance, what the server cannot hold left out with a warning.
- Fixed a Google series written in another zone than its calendar's drifting an hour across a daylight-saving change.
- Fixed a Google or Outlook account refusing an event with neither `DTEND` nor `DURATION`: it ends at its start, or the next day after a date start (RFC 5545 3.6.1).
- Fixed `event add --attendee` on a Google account mailing no invitation: an event the account organizes is inserted with `sendUpdates=all`, keeping its `UID` ([#20](https://github.com/pimalaya/calendula/issues/20)).
- Fixed `event read` of a deleted Google event printing its cancelled stub: it fails `item-not-found`, and so does an update over it.
- Fixed a Google calendar or event id holding `#`, `?` or `/` naming another resource, so a subscribed calendar such as `en.usa#holiday@group.v.calendar.google.com` lists.
- Fixed a Google listing failing whole on a value calendula does not know: it prints it as the API spells it, and an update leaves it as the server has it.
- Fixed an update of a Google event whose reminders are off turning the calendar's default reminders back on.
- Fixed a Google `calendar update -n New -C ""` renaming the calendar before refusing the cleared colour, and a Google `calendar create --color` whose paint failed leaving a calendar behind.
- Fixed `gcal acl create` posting a `user`, `group` or `domain` scope with no value, and a `default` scope dropping its `--value` and granting everyone: both are refused.
- Fixed an Outlook series ending a day off away from UTC.
- Fixed an Outlook calendar or event id holding `/`, `?`, `#` or `%` naming another resource.
- Fixed an Outlook event's alarm that Graph's reminder cannot hold coming back scattered across the event after a write.
- Fixed an unset Outlook calendar colour printing `""`: it prints `null`.
- Fixed a folded line Google or Graph does not model coming back damaged after a write.
- Fixed a CalDAV `event update` or `item update` given a source creating an item under an id the calendar did not hold: it fails `item-not-found`.
- Fixed a CalDAV calendar or item id that is empty or holds an empty, `.` or `..` segment reaching the server, where `calendar delete ..` could delete the home set's parent: it fails before connecting.
- Fixed a CalDAV calendar colour written as the inf-it `calendar-color`, which iCloud ignores: it is written and read as Apple's, the inf-it one still read.
- Fixed `calendar update -d ""` keeping the description on CalDAV and Google while reporting success, and `-C ""` writing nothing on CalDAV.
- Fixed a CalDAV to-do or journal entry named by a digest: a create names it after its `UID`, as an event.
- Fixed a CalDAV item created from a `UID` holding `@` printing another id than the listings: the resource name is percent-encoded (`%40`).
- Fixed a CalDAV item read through a listing losing its final CRLF, so it matches the body `item read` returns.
- Fixed a CalDAV weak entity tag losing its closing quote, in a listing and in `--if-match`.
- Fixed a `caldav.server` with a scheme other than `http` or `https`, or one failing to parse, read as a bare host: it is refused.
- Fixed a `caldav` block naming both `server` and `home` loading with `home` winning silently: it is refused.
- Fixed `account check` passing a CalDAV account with a pinned `caldav.home` whatever its credential: it sends a PROPFIND on the home.
- Fixed an HTTP answer preceded by an interim `1xx` response read as that response, and a chunked body's trailers starting the next answer of a reused connection.
- Fixed a vdir calendar that does not exist read as an empty one, `event add -k nope` creating it: every command naming one fails `calendar-not-found`.
- Fixed a vdir calendar id escaping the home, where `calendar delete -k ..` removed the directory holding it.

  An id that is not one directory name (empty, `.`, `..`, holding a separator) is not found, and refused by `calendar create` and `vdir create`.

- Fixed `vdir rename` onto an existing calendar failing with a raw OS error, or replacing an empty directory: it fails `calendar-duplicate`, nothing moved.
- Fixed a vdir listing failing on a directory or a dangling link named like an item, or on an item deleted while it lists.
- Fixed `event agenda` placing a recurring series once, at its first start: it expands each series over the days drawn.
- Fixed `event agenda` reading a lone month name as a year (`event agenda noviembre`), and a value it could not read as a default: it is refused, naming the value.
- Fixed `event agenda` stamping an event written in UTC at its UTC time: it files each event in the local zone.
- Fixed `event agenda` leaving out the timed events within the zone's UTC offset of its grid's edges.
- Fixed the composed-item check refusing a whole-day, a zoned or an organized event, all of which RFC 5545 allows.
- Fixed a `TZID` with a leading slash (`/Europe/Paris`) read in the local zone rather than its own.
- Fixed a `VTIMEZONE` stating no observance read as UTC: the time is read in the local zone, with `zoneAssumed`.
- Fixed an `EXDATE` or a `RECURRENCE-ID` written in UTC on a zoned series missing its occurrence, the excluded one still listing and the override listing beside the one it replaced.
- Fixed the `event delete` help naming the iCalendar `UID`: it takes the id `event list` shows, a `UID` failing `item-not-found` naming `event read --uid`.
- Fixed `event read` and `item read` printing the stored iCalendar's control characters, which could drive the terminal: the text replaces them, `--json` keeping the stored text.
- Fixed a credential command prompting for a password (`systemd-ask-password`) hanging the run: it reads the terminal.
- Fixed a credential command run again each time a command reopened its backend: it runs once per account.
- Fixed install.sh saving an HTTP error page as the archive: it stops on the error.

### Security

- Fixed the composer's draft being created with the umask's permissions, or through a file or a link another user planted at its path: it is created afresh, readable by its owner alone on Unix.
- Fixed the trace log showing sensitive response headers, and the PIN, passcode and password of a Google conference entry point.
- Bumped rustls to 0.23.46 and rustls-webpki to 0.103.17, fixing RUSTSEC-2026-0346 to RUSTSEC-2026-0349, RUSTSEC-2026-0382 and RUSTSEC-2026-0383.

## [0.2.0] - 2026-10-02

### Added

- Added the `gcal` backend, calendula over the Google Calendar API v3 through [io-gcal](https://github.com/pimalaya/io-gcal), behind a `gcal` feature and an `[accounts.<name>.gcal]` block carrying a bearer `auth.token`.

  Google's CalDAV bridge is crippled, so a Google account used to be read-mostly with no calendar creation. Calendars are now writable, a date range is pushed down as `timeMin` and `timeMax`, a listing walks `nextPageToken` only as far as the requested page, and an update honours `If-Match`.

  Google exposes no iCalendar, so the backend synthesizes the document and re-projects it on write. Managed fields go both ways, Google-only fields stay untouched by an update, provider-scoped ones are minted read-only as `X-GOOGLE-*`, and every other line is stashed in `extendedProperties.private`.

  A zoned boundary carries the VTIMEZONE it references, so an item stands on its own as an .ics file and a series keeps expanding where it did across daylight-saving changes. The feature therefore carries a time zone database.

  A recurring series and its modified instances are one item, the exceptions folded into the master's document under its UID with a RECURRENCE-ID, as RFC 4791 4.1 requires. Only VEVENT projects: a VTODO or VJOURNAL is refused by name.

- Added the `gcal` command family for what iCalendar cannot express: `calendars`, `acl`, `free-busy`, `instances`, `move`, `quick-add`, `colors` and `settings`.

  Push channels, `calendars.clear` and `transferOwnership` are deliberately absent.

- Added the Microsoft Graph backend, `msgraph`, in the default feature set, for Outlook and Microsoft 365 calendars.

  Events are synthesized as iCalendar both ways by io-msgraph's `ical` feature, and a recurring series with its exceptions is one item. Graph keeps no calendar description nor RGB colour, so naming either is refused, and only the series master is written back.

- Added the `pimdir` backend, calendula over a local [pimdir](https://github.com/pimalaya/pimdir) store, behind a `pimdir` feature and an `[accounts.<name>.pimdir]` block carrying a `root` and an optional `account`.

  It never owns the store. It reads through the lock-free reader, so a listing runs beside a sync, and stages each write as one queue action. Calendars come from the sync, so `calendar create`, `update` and `delete` refuse.

  The reader folds the pending queue over its reads, so a staged edit or deletion shows at once. A staged creation has no public id until the sync applies it, so it is counted rather than listed.

  Reads are availability-aware: an item the sync has not downloaded still lists, and reading it reports "body not fetched". Its kind and date come from the typed summary row the store's owner derives (pimdir STORAGE Annex A).

  Link ids, body hashes and summaries come from io-pimdir's own derivations, so an item calendula stages and the same item arriving through a sync are one item. Two resources whose bodies carry one UID both list, each under its own public id.

  A store written by an earlier io-pimdir draft is refused, with a message asking to delete it and resync.

- Added `pimdir status`, reporting the account being read, every account the store groups collections under, how much of each calendar is downloaded, and how many creations are queued.

- Added a command family per component kind: `todo` (VTODO) and `journal` (VJOURNAL) join `event` (VEVENT), each with `list`, `read`, `build`, `create`, `update` and `delete`.

  They are views over the same items, rendering the columns their kind is read by, and add no backend operation. `--from` and `--to` narrow them locally, a server-side range being defined against a start and an end neither kind carries. VFREEBUSY and VTIMEZONE get no family.

- Added a composer to `build`, `create` and `update`, opened by `-i/--interactive` and configured by `item.composer`.

  `item.composer` is a shell line or an argv list, at the top level and per account. `--composer <COMMAND>` overrides it for one run and requires `-i`, and `--json` refuses to spawn one.

  It is spawned on a temporary iCalendar file with every stream inherited. Changed bytes are the item, an emptied or untouched file abandons the edit, and a non-zero exit is a failure.

  What a composer wrote is validated against RFC 5545 first, a re-edit being offered on violations, and a composed item that cannot be written keeps its file and names it. An iCalendar given on the command line is never checked.

- Added `build` to the four families, the create pipeline stopped before the write.

  It prints the iCalendar, reaches no backend and reads no configuration unless `-i` needs the composer. `-o/--output <PATH>` captures it, `-i` owning stdout, and an abandoned build prints nothing at exit 0.

- Added `event agenda`, a cal(1)-style grid marking the days that carry an event, listed underneath by start.

  Every event at one instant is rendered under a single time column, and the `--json` payload maps each start to the labels starting at it.

- Added `--from` and `--to` to `event list` (YYYY-MM-DD, both inclusive).

  They are pushed down as an RFC 4791 `time-range` filter on CalDAV and applied after parsing elsewhere, pimdir answering from the stored summary. A range lifts the default page-size cap.

- Added `-b/--backend`, selecting which backend the shared commands target.

  `auto` takes the first configured one in priority order: vdir, pimdir, caldav, gcal, msgraph.

- Added the `json-schema` command (alias `json-schemas`), printing the JSON Schema of a command's `--json` payload, or writing one file per command under `--dir`.

- Added `gcal.alpn`, the ALPN identifiers offered during the TLS handshake.

  Unset offers `http/1.1`, and an empty list skips negotiation. Only rustls reads it.

### Changed

- **BREAKING**: switched every `--json` payload key to camelCase, and made the four `read` commands answer `{"contents"}` rather than `{"message"}`.

  `caldav discover` renamed `calendar_home_set` to `calendarHomeSet`, and `vdir list` renamed `display_name` to `displayName`. The configuration stays kebab-case, and every command's plain-text output is unchanged.

- **BREAKING**: made item ids the resource names a CalDAV server returned, verbatim, io-webdav no longer appending nor stripping `.ics`.

  It fixes read, update and delete addressing the wrong resource whenever an id did not end in `.ics`, and a create returning an unusable id when the server named the resource itself. Scripts pinning a hand-built id need updating.

- **BREAKING**: rewrote the wizard on the Himalaya model as `calendula configure` (alias `wizard`), replacing `account configure`.

  One prompt takes an email address, a server URL or a local folder path, and its shape orients the rest. The account name is derived rather than prompted, and the account is tested before anything is emitted.

  It is offered where nothing can happen without a configuration, and skipped under `--json` or a non-terminal stdin. The account is written to a missing file, appended as plain text to an existing one so comments survive, or printed.

- **BREAKING**: renamed the remote backend from `webdav` to `caldav` across the cargo feature, the subcommand and the config block.

  Only the io-webdav dependency keeps the WebDAV name.

- **BREAKING**: dropped the io-calendar dependency, moving the cross-protocol layer into calendula, one adapter per protocol, as cardamum does.

  io-calendar is frozen and pinned io-vdir 0.0.3 and io-webdav 0.0.1, so nothing below it could move while it stayed.

- Renamed `completions` and `manuals` to `completion` and `manual`, and the shared subcommands to the singular `calendar`, `event` and `item`, the plurals staying as hidden aliases.

  The `gcal` commands mirroring a Calendar API resource keep that API's spelling and gained hidden singular aliases.

- Made the iCalendar of `create` and `update` an optional trailing positional on all four families: a path, raw contents, or `-` for stdin.

  Both bail when given neither a source nor `-i`. A create with no source mints its item, carrying a `PRODID`, a `UID`, a `DTSTAMP` and, on a VEVENT, a `DTSTART`.

  An update with no source starts from the stored item and sends the entity tag it read as `If-Match`, unless `--if-match` names another. `item` names no component kind, so it mints nothing and says which families do.

- Made `create` and `update` answer a named output type rather than the printer's `Message`, an abandoned edit being an outcome they report.

  Both are untagged, so a write serializes as `{"id"}`.

- Made the parent calendar an optional `-k/--calendar` flag on every shared command instead of a positional, falling back to the new `calendar.default`.

  `calendar delete` takes it as a mandatory flag that never falls back.

- Deferred opening the backend connection to the first call that needs it.

  A command running a composer holds none open while the editor is up, where a write after a long edit used to read a socket the server had closed. The four families resolve their account per subcommand, which lets `build` resolve none.

- Made `calendar create` report the identifier the backend assigned rather than the one asked for.

  They differ only on Google, which mints its own.

- Made `caldav.server` take a bare authority (`dav.example.org`, `domain:port`) as well as a full URL, defaulting to `https`, as the sibling products spell the same key.

- Resolved each account credential once, so a `pass` or `gpg` entry shared by a `caldav` and a `gcal` block is unlocked once.

- Stopped writing a `default = false` line in wizard-generated accounts.

- Adopted [ical-rs](https://github.com/pimalaya/ical) for iCalendar parsing, an item whose bytes do not parse being skipped rather than failing the listing.

- Bumped the Pimalaya dependencies to their current releases, and turned the `pimdir` feature on by default.

  ical-rs 0.5, io-http 0.5, io-pimdir 0.5, io-vdir 0.2, io-webdav 0.5, pimalaya-cli 0.2, pimalaya-config 0.2, pimalaya-stream 0.3, and pimconf's successor io-pim-discovery 0.8.

- Made a blocking stream that reports it is not ready retry for a minute before giving up (pimalaya-stream 0.3). **Behaviour change.**

  A read deadline is armed at connect time. An exchange that used to end with a bare `Resource temporarily unavailable (os error 35)`, on macOS especially, now survives it.

- Turned `vendored` on by default and forwarded it to io-pimdir, so `cargo install` builds SQLite from source and needs none on the machine.

  Drop it to link the system SQLite and save about 1 MB. It also vendors OpenSSL when `native-tls` is on. The Nix builds still link the store's SQLite.

- Replaced the direct comfy-table dependency with pimalaya-cli's re-export, moving it to v8.

  `table.preset` keeps its v7 positional string, mapped onto the new typed style. The truncation indicator changes from `...` to `…`.

- Included the affected calendar or collection id in every `calendar`, `caldav` and `vdir` success message.

- Documented each command's JSON output shape as the last paragraph of its `--help`, and slimmed the README Usage section to a pointer.

- Relicensed from AGPL-3.0-only to dual MIT OR Apache-2.0.

### Removed

- Removed `downloads-dir`, read by nothing: calendula downloads no attachment.

### Fixed

- Fixed a component listing paginating before it narrowed by kind, so a page of `todo list` could come back empty.

  A calendar of thirty events and one todo answered `todo list` with an empty table. CalDAV now pushes the kind down as an RFC 4791 `comp-filter` with the window nested inside it, gcal answers a non-VEVENT kind without a round-trip, and pimdir reads the kind off the stored summary.

- Fixed a composed VEVENT carrying no DTSTART being sent, now refused.

  RFC 5545 3.6.1 requires it unless the calendar specifies a METHOD, and SabreDAV answered HTTP 500 rather than naming it.

- Fixed a whitespace-only source being handed to a backend as an empty body, now refused naming where it was read from.

- Fixed CalDAV discovery settling on a URL with no DAV server behind it, such as a web page (io-pim-discovery 0.8).

  Every candidate must now answer a `PROPFIND`, and discovery fails otherwise rather than configure it.

- Fixed `--config` not reaching the wizard, the one path where a user is most likely to pass it.

- Fixed `tls.cert` not expanding `~` and environment variables, `cert = "~/ca.pem"` looking for a literal `./~/ca.pem`.

  Every configured path is now expanded when it is read rather than at each use.

- Fixed the pimdir backend refusing an unknown calendar without listing the ones it can take, a namespaced collection id being impossible to guess.

- Fixed `completion` writing files to the working directory instead of printing the script to stdout, which broke every packaging helper capturing stdout.

  `manual` shares its shape, a positional list of command names and an optional `--dir`, so `calendula manuals ./man` becomes `calendula manual --dir ./man`.

- Fixed a CalDAV calendar's time zone being dropped on create and update, io-webdav reading `calendar-timezone` when listing but never writing it back.

- Fixed an item listing keeping a calendar's own multistatus self-entry, which iCloud echoes, as a bogus item.

- Fixed the missing `Host` header on HTTP/1.1 requests (io-http 0.2), which made servers answer 400 and broke the `.well-known` discovery probe.

- Fixed `tls.cert` for a self-signed server (pimalaya-stream 0.1.1), the certificate now being pinned to the server's leaf instead of registered as a trust anchor that rejected it.

## [0.1.0] - 2025-10-27

### Added

- Add date column in item listing

### Changed

- Init code from Cardamum CLI
- Rename properly commands, variables and docs

### Fixed

- Fix wrong AGPL license

## [root] - 2025-10-25

### Added

- Init repository

[Unreleased]: https://github.com/pimalaya/calendula/compare/v0.2.0..HEAD
[0.2.0]: https://github.com/pimalaya/calendula/compare/v0.1.0..v0.2.0
[0.1.0]: https://github.com/pimalaya/calendula/compare/root..v0.1.0

<!-- generated by git-cliff on 2025-10-27T20:52:53.305850512+01:00 -->
