# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added a shared command family per iCalendar component kind: `todo` (VTODO) and `journal` (VJOURNAL) join `event` (VEVENT), each with `list`, `read`, `create`, `update` and `delete`.

  A todo listing renders the summary, due date, status, priority and completion percentage a task list is read by; a journal listing renders the summary, date and status of a dated note.

  They are views over the same items, so they add no backend operation, and `item` stays the raw unfiltered one.

  `--from` / `--to` narrow them, applied after parsing rather than pushed down: a server-side range filter is defined against a component's start and end, which a todo and a journal entry do not both carry.

  VFREEBUSY and VTIMEZONE get no family, the first being the answer to a query and the second the definition of the zones the others reference.

- Added the `gcal` command family, covering the half of the Calendar API that iCalendar cannot express: `calendars`, `acl` (`list`, `create`, `update`, `delete`), `free-busy`, `instances`, `move`, `quick-add`, `colors` and `settings`.

  `calendars` is the richer listing, carrying the access role, the primary flag, the time zone and the default reminders.

  Push channels are deliberately absent: a channel delivers to an HTTPS endpoint the caller must host, which a CLI has not. So are `calendars.clear` and `transferOwnership`, both irreversible.

- Added the `gcal` backend: calendula over the Google Calendar API v3, through [io-gcal](https://github.com/pimalaya/io-gcal).

  Google is reachable over CalDAV only in a crippled form (bearer tokens only, `MKCALENDAR` refused, an off-spec discovery entry point), so a Google account was read-mostly and calendar creation impossible.

  The native backend sits behind a `gcal` cargo feature and an `[accounts.<name>.gcal]` block carrying a bearer `auth.token` secret, which an OAuth 2.0 token broker fills like any other command.

  Calendars can now be created, updated and deleted, a date range is pushed down as `timeMin` / `timeMax`, a listing walks `nextPageToken` only as far as the requested page, and an update honours `if_match` through `If-Match`.

  Google stores a JSON event and exposes no per-event iCalendar representation, so the backend synthesizes the document of record and re-projects it on write.

  Fields with a well-defined iCalendar slot are managed both ways, Google-only fields survive an update untouched, provider-scoped ones are minted as read-only `X-GOOGLE-*` properties, and every remaining line is stashed verbatim in `extendedProperties.private`.

  Only VEVENT projects: a VTODO or VJOURNAL is refused by name, since Google models neither.

  Google returns a boundary as an absolute instant plus the calendar's display zone, so only the offset decides the instant and the zone is carried over from the server copy on update.

  That keeps a zoned recurring series expanding where it did, rather than falling back to UTC and drifting by an hour after a daylight-saving change.

  A boundary anchored in a named zone carries the VTIMEZONE it references, minted from the zone name Google sends, so an item stands on its own as an .ics file. The Calendar API carries the IANA name and nothing behind it, so the `gcal` feature carries a time zone database of its own.

  A recurring series and its modified instances are **one** item rather than several.

  Google is instance-granular where iCalendar is resource-granular: it hands an exception over as an event of its own, so the backend folds it back into the master's document, under the master's UID and with the RECURRENCE-ID naming the instance it replaces.

  That is the single calendar object resource RFC 4791 4.1 requires, and it is what a CalDAV store of the same calendar holds, so the two agree on how many items exist.

  Reading a series costs one extra listing, filtered by its iCalUID, since the API offers no query for the children of an event.

- `calendar create` now reports the identifier the backend assigned rather than the one asked for.

  They differ only on Google, which mints its own, and the reported id is the one later commands address the calendar by.

- Added the `pimdir` backend: calendula over a local [pimdir](https://github.com/pimalaya/pimdir) store, the offline cache a sync engine fills.

  It sits behind a `pimdir` cargo feature and a `[accounts.<name>.pimdir]` block carrying a `root` and an optional `account`.

  Reads are availability-aware: an item the sync listed but has not downloaded still shows in a listing, and reading it reports "body not fetched" rather than failing. Calendars come from the sync, so `calendar create`, `update` and `delete` refuse here.

  calendula takes the two roles the format gives a consumer of a store it does not own (pimdir SPEC 8). It reads through the lock-free reader, so a listing runs beside a sync instead of locking it out.

  It writes through the enqueue-only producer, appending one queue action per write for the sync to apply and push.

  The reader folds the pending queue over its reads, so a staged edit or deletion shows straight away; a staged creation has no public id until the sync applies it, and is counted rather than listed.

  An item's link id, `v: 1` summary and body hash come from io-pimdir's own derivations, so an item calendula stages and the same item arriving through a sync are one item rather than two.

  The summary is the pimdir SPEC Annex A.3 `text/calendar` convention: the resource's UID, its component, its location, DTSTART with the TZID naming its zone and the value type saying whether it has a time at all, DTEND, DUE, whether the item recurs and, when bounded, its UNTIL.

  That UNTIL brackets the series, so a range read drops it without expanding an occurrence. calendula reads the summary to answer a date question on an item whose body is not local.

  A listing comes back in the store's own calendar order, the sort key ascending, which is a resolved start. A staged action carries no key: the format leaves it to the sync that pushes the write, and a producer deriving one would order an item the connector is about to reorder.

  A calendar may hold two resources whose bodies carry one UID, which RFC 4791 4.1 forbids and servers do not always enforce, and both list: the store keys them apart (pimdir SPEC 9) and draws each its own public id, so an item is addressed by that id and never by the identity its body states.

  The two copies need not be the same event, so neither is hidden behind the other.

- Added `pimdir status`, reporting the account being read, every account the store groups collections under, how many of each calendar's items carry a local body, and how many creations are queued for the next sync.

- Added `event agenda`, a cal(1)-style grid marking the days that carry an event, listed underneath by start.

  Every event starting at one instant is rendered, ordered by label then by item id, under a single time column, so two meetings at 09:00 read as two lines. The `--json` payload maps each start datetime to the list of the labels starting at it, carrying the same multiplicity.

- Added `--from` / `--to` date-range filtering to `event list` (YYYY-MM-DD, both inclusive).

  CalDAV pushes it server-side as an RFC 4791 `time-range` filter; the local backends apply it after parsing, and pimdir answers it from the stored summary when the body is not local. A range also lifts the default page-size cap, so every match is returned.

- Added the `-b/--backend` flag selecting which backend the shared commands target. The default, `auto`, takes the first configured one in calendula's priority order (vdir, pimdir, caldav, gcal).

- Adopted the [Cairn](https://github.com/pimalaya/cairn) convention: cairn/spec holds the living specification (backends, commands, config, wizard, packaging), cairn/changes the proposals, cairn/log the dated history.

- Added `json-schema`, printing the JSON Schema of a command's `--json` payload (aliased `json-schemas`).

  Every data command now hands the printer a named `*Output` type registered in the schema map, so a consumer can validate what it reads instead of guessing. Passing `--dir` writes one file per command; passing none prints the single schema named on the command line.

- Added `gcal.alpn`, the ALPN identifiers offered during the Google Calendar TLS handshake.

  Unset offers `http/1.1`, the only HTTP version io-http speaks; an empty list skips ALPN negotiation and a non-empty one replaces the default. Only rustls reads it, native-tls ignoring ALPN altogether.

### Changed

- A credential command named by two backends of one account is now run once per account instead of once per backend.

  `account check` and the wizard's connection test both exercise every backend the account configures, so a `pass` or `gpg` entry shared by its `caldav` and `gcal` blocks was unlocked twice. One resolver now serves the whole check, hands the value of a command it already ran to every backend naming it, and is dropped with the check.

- `--config` now reaches the wizard.

  It was passed to every subcommand and dropped on the one path where a user is most likely to pass it, so a wizard run under `--config <path>` neither read nor wrote that path.

- **BREAKING**: renamed `completions` and `manuals` to `completion` and `manual`, the plural staying as a hidden alias.

  The `gcal` commands mirroring a Calendar API resource keep that API's spelling (`calendars`, `instances`, `colors`, `settings`) and gained hidden singular aliases.

- Bumped io-http to 0.5.

- Bumped pimalaya-stream to 0.3, whose `Read` and `Write` retry a stream reporting it is not ready. **Behaviour change.**

  A blocking socket is not supposed to report `EAGAIN`, yet callers saw one surface mid-exchange and end the exchange with a bare `Resource temporarily unavailable (os error 35)`, macOS especially and the more readily the longer the exchange ran.

  The transport now retries such a failure for a minute before giving up with a `TimedOut` naming the budget, and arms a socket read deadline at connect time so a server going silent on a healthy connection stops blocking the caller forever.

  Its `StreamStd` is renamed `stream::Stream` and its connects take a per-transport options struct, which is what this crate now calls.

- Bumped pimalaya-stream to 0.2, whose only change here is the removal of its SASL module: this crate uses the TLS options and the blocking stream, neither of which moved.

- **BREAKING** Rewrote the wizard on the Himalaya model, as `calendula configure` (alias `wizard`).

  One prompt now takes an email address, a server URL or a local folder path, and its shape orients the rest: an address runs bounded parallel discovery and each reachable server becomes one entry, a URL is taken as the CalDAV context root, a folder is detected as a vdir home or a pimdir store.

  The account name is derived from the input rather than prompted, and the account is tested before anything is emitted.

  It runs when you ask for it, and it is offered where nothing can happen without a configuration: a bare `calendula` finding none, and a command needing an account finding none.

  A bare `calendula` finding one prints the help, as does one carrying `--account`, and the offer is skipped in JSON mode and whenever stdin is not a terminal.

  The generated account is saved to a configuration file that does not exist yet, appended as plain text to one that does so comments and formatting survive, or printed; its name is suffixed until free, and it claims the default only when no account already does.

- **BREAKING** Removed `account configure`, replaced by the top-level `calendula configure`, which is where Himalaya and Cardamum put theirs.

- **BREAKING** Dropped the io-calendar dependency and moved the cross-protocol layer into calendula.

  Following the cardamum precedent, the shared types and the backend dispatcher are the product's own, with one adapter per protocol. io-calendar is frozen and still pinned io-vdir 0.0.3 and io-webdav 0.0.1, so nothing below it could move while it stayed.

- **BREAKING** Item ids are now the resource names a CalDAV server returned, verbatim.

  io-webdav no longer appends nor strips a `.ics` extension, which fixes item read, update and delete addressing the wrong resource whenever an id did not end in `.ics`, and a create returning an unusable id when the server named the resource itself.

  An id a listing shows now round-trips through every verb. Scripts pinning a hand-built id need updating.

- Adopted [ical-rs](https://github.com/pimalaya/ical) for iCalendar parsing.

  io-calendar's `as_ical` went with the crate and io-vdir 0.1 removed its own parser, so the VEVENT projection behind `event list` and `event agenda` now lives in one place. An item whose bytes do not parse is skipped rather than failing the whole listing.

- Bumped every remaining Pimalaya dependency to its current release: io-vdir 0.1, io-http 0.3, pimalaya-cli 0.2, pimalaya-config 0.1.1, pimalaya-stream 0.1.2, and pimconf to its renamed successor io-pim-discovery 0.5.

- Replaced the direct comfy-table dependency with pimalaya-cli's re-export, which moves it to v8.

  The `table.preset` option keeps its v7 positional string, mapped onto the new typed style, so existing configurations stay valid; the default truncation indicator changes from `...` to `…`.

- Replaced the root ARCHITECTURE.md with the src/main.rs header and the cairn specification, following the org convention that retired the per-repo architecture document.

- Documented each command's JSON output shape as the last paragraph of its `--help` text, and slimmed the README Usage section down to a pointer to `calendula --help`.

- Extracted the `-k/--calendar CALENDAR-ID` flag into a shared argument reused across the whole shared API.

  `calendar update` takes it (replacing its positional id, with the usual `calendar.default` fallback) and `calendar delete` takes it as a mandatory flag that never falls back.

- Made the parent calendar of every `event` and `item` command an optional `-k/--calendar CALENDAR-ID` flag instead of a positional argument; when omitted it falls back to the new `calendar.default` config, otherwise the command bails.

- Changed `event create`/`event update` and `item create`/`item update` to take their iCalendar as a trailing positional `ICAL` argument (a path, raw iCalendar contents, or `-` for stdin) instead of reading only from stdin, matching `tcal edit`.

- Included the affected calendar or collection id in every `calendar`, `caldav` and `vdir` success message.

- Migrated to the pimalaya-cli / pimalaya-config / pimalaya-stream stack and adopted the Himalaya v2 CLI structure.

- Renamed the shared subcommands to the singular `calendar`, `event` and `item` to match Himalaya; the plural forms stay as hidden aliases.

- Renamed the remote backend from `webdav` to `caldav` across the public surface: the cargo feature, the subcommand and the config block. Only the underlying io-webdav dependency keeps the WebDAV name.

- Relicensed from AGPL-3.0-only to dual MIT OR Apache-2.0.

- `caldav.server` now takes a bare authority as well as a full URL.

  A bare domain (`dav.example.org`) or a `domain:port` pair is accepted and defaults to `https`, which is how the sibling products spell the same key. Every URL that worked before still works.

- **BREAKING**: the four `read` commands answer `{"contents": "..."}` instead of `{"message": "..."}` under `--json`.

  `Message` carries a confirmation, not data, so `event read`, `todo read`, `journal read` and `item read` now return a payload of their own. The plain-text output is unchanged.

- **BREAKING**: every `--json` payload key is camelCase now.

  `caldav discover` renamed its `calendar_home_set` key to `calendarHomeSet` and `vdir list` its `display_name` to `displayName`; the keys added this cycle read `syncToken`, `timeZone`, `accessRole`, `defaultReminders`, `originalStart`, `calendarId` and `percentComplete`. Every other key is a single word and is unchanged.

  That is the spelling of the wire formats calendula sits over, the Calendar API answering `accessRole` and `timeZone`, and it is the one shape a jq path addresses without quoting.

  The configuration vocabulary stays kebab-case: a TOML document is written by a person, not by a script. The plain-text output of every command is unchanged.

- A wizard-generated account no longer writes a `default = false` line, the rest of the family omitting it too.

### Removed

- Removed the `downloads-dir` option, which was read by nothing.

  It was declared globally and per account, merged into the runtime account and exposed through an accessor no command called. calendula downloads no attachment, so there was nothing to point it at.

### Fixed

- Fixed `tls.cert` never expanding a `~` or an environment variable, so `cert = "~/ca.pem"` looked for a literal `./~/ca.pem` under the working directory.

  Every path the configuration carries is now expanded when it is read rather than at each use, which is what `vdir.home-dir` and `pimdir.root` already did by hand at five separate call sites.

- Fixed the pimdir backend refusing an unknown calendar without saying which ones the account holds.

  A calendar is its collection id, which carries the namespace the sync engine binds it under, so the id to type is not guessable and the refusal now lists the ones it can take.

- Fixed `completions` writing files to the working directory instead of printing the script to the standard output, which broke every packaging helper capturing stdout.

  `manuals` now shares its shape: a positional list selecting what to generate, printed to stdout, and an optional `--dir` deciding where it lands instead of the directory it used to take as a positional argument.

  `calendula manuals ./man` becomes `calendula manuals --dir ./man`, and both accept command names (`calendula`, `calendula-event`) to generate a single item.

- Fixed a CalDAV calendar's time zone being silently dropped on create and update: io-webdav read `calendar-timezone` when listing but never wrote it back.

- Fixed an item listing keeping a calendar's own multistatus self-entry as a bogus item, which iCloud echoes.

- Fixed the missing `Host` header on HTTP/1.1 requests (through io-http 0.2), which made servers answer 400 and silently broke the `.well-known` discovery probe.

- Fixed `tls.cert` being unusable for a self-signed server (through pimalaya-stream 0.1.1): the certificate is now pinned to the server's leaf instead of being registered as a trust anchor that rejected it.

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

[0.1.0]: https://github.com/pimalaya/ortie/compare/root..v0.1.0

<!-- generated by git-cliff on 2025-10-27T20:52:53.305850512+01:00 -->
