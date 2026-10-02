# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added the Microsoft Graph backend, `msgraph`, in the default feature set: Outlook and Microsoft 365 calendars through the Graph API, events synthesized as iCalendar both ways by io-msgraph's `ical` feature, a recurring series with its exceptions as one item.

  Graph keeps no calendar description nor RGB colour, so naming either is refused; only the series master is written back.

- Added a composer, the command `item.composer` names, opened by `-i/--interactive` on `build`, `create` and `update`: it is spawned on the path of a temporary iCalendar file with every stream inherited, and what it leaves there is the decision. Changed bytes are the item, an emptied or untouched file is an edit given up on, and a non-zero exit is a failure. `--composer <COMMAND>` overrides it for one run and requires `-i`; `--json` refuses to spawn one. What a composer wrote is validated against RFC 5545 first, printing its violations and offering a re-edit, and a composed item that cannot be written keeps its file and names it in the error. An iCalendar given on the command line is never checked.
- Added `build` to the four families, the create pipeline stopped before the write: it prints the iCalendar, reaches no backend and reads no configuration unless `-i` needs the configured composer. `-o/--output <PATH>` captures it, `-i` owning stdout, and an abandoned build prints nothing at exit 0.
- Added `item.composer`, a shell line or an argv list, at the top level and per account.
- Added a command family per component kind: `todo` (VTODO) and `journal` (VJOURNAL) join `event` (VEVENT), each with `list`, `read`, `build`, `create`, `update` and `delete`, rendering the columns its kind is read by. They are views over the same items and add no backend operation; `--from` / `--to` narrow them locally, a server-side range filter being defined against a start and an end neither kind carries. VFREEBUSY and VTIMEZONE get no family.
- Added the `gcal` backend, calendula over the Google Calendar API v3 through [io-gcal](https://github.com/pimalaya/io-gcal), behind a `gcal` feature and an `[accounts.<name>.gcal]` block carrying a bearer `auth.token`. Google's CalDAV bridge is crippled, so a Google account was read-mostly and calendar creation impossible; calendars are now writable, a date range is pushed down as `timeMin` / `timeMax`, a listing walks `nextPageToken` only as far as the requested page, and an update honours `If-Match`.
- The `gcal` backend synthesizes the iCalendar document Google exposes nowhere and re-projects it on write: managed fields both ways, Google-only fields untouched by an update, provider-scoped ones minted as read-only `X-GOOGLE-*`, and every other line stashed in `extendedProperties.private`. Only VEVENT projects, a VTODO or VJOURNAL being refused by name.
- A zoned `gcal` boundary carries the VTIMEZONE it references, minted from the zone name Google sends, so an item stands on its own as an .ics file and a recurring series keeps expanding where it did instead of drifting after a daylight-saving change. The `gcal` feature therefore carries a time zone database.
- A `gcal` recurring series and its modified instances are one item, the exception folded back into the master's document under the master's UID with a RECURRENCE-ID, which is the single resource RFC 4791 4.1 requires and what a CalDAV store of the same calendar holds.
- Added the `gcal` command family for the half of the API iCalendar cannot express: `calendars`, `acl`, `free-busy`, `instances`, `move`, `quick-add`, `colors` and `settings`. Push channels, `calendars.clear` and `transferOwnership` are deliberately absent.
- Added the `pimdir` backend, calendula over a local [pimdir](https://github.com/pimalaya/pimdir) store, behind a `pimdir` feature and a `[accounts.<name>.pimdir]` block carrying a `root` and an optional `account`. It takes the two roles the format gives a consumer that does not own the store: it reads through the lock-free reader, so a listing runs beside a sync, and stages each write as one queue action.
- The pimdir reader folds the pending queue over its reads, so a staged edit or deletion shows straight away; a staged creation has no public id until the sync applies it and is counted rather than listed. Reads are availability-aware: an item the sync has not downloaded still lists, and reading it reports "body not fetched". Calendars come from the sync, so `calendar create`, `update` and `delete` refuse.
- A pimdir item's link id and body hash come from io-pimdir's own derivations, and its summary is the typed event, task or journal row the store's owner derives from the staged body (pimdir STORAGE Annex A), so an item calendula stages and the same item arriving through a sync are one item. That row answers the kind and the date of an item whose body is not local. A store written by an earlier io-pimdir draft is refused with a message asking to delete it and resync.
- Two pimdir resources whose bodies carry one UID both list, each under its own public id: the store keys them apart, they need not be the same event, and an item is addressed by that id rather than by the identity its body states.
- Added `pimdir status`, reporting the account being read, every account the store groups collections under, how much of each calendar is downloaded, and how many creations are queued.
- Added `event agenda`, a cal(1)-style grid marking the days that carry an event, listed underneath by start. Every event at one instant is rendered under a single time column, and the `--json` payload maps each start to the labels starting at it.
- Added `--from` / `--to` to `event list` (YYYY-MM-DD, both inclusive), pushed down as an RFC 4791 `time-range` filter on CalDAV and applied after parsing elsewhere, pimdir answering from the stored summary. A range lifts the default page-size cap.
- Added `-b/--backend`, selecting which backend the shared commands target; `auto` takes the first configured one in priority order (vdir, pimdir, caldav, gcal).
- Added `json-schema` (aliased `json-schemas`), printing the JSON Schema of a command's `--json` payload, or writing one file per command under `--dir`.
- Added `gcal.alpn`, the ALPN identifiers offered during the TLS handshake. Unset offers `http/1.1`, an empty list skips negotiation; only rustls reads it.
- Adopted the [Cairn](https://github.com/pimalaya/cairn) convention: cairn/spec holds the living specification, cairn/changes the proposals, cairn/log the dated history.

### Changed

- Forwarded `vendored` to io-pimdir and turned it on by default, so `cargo install` builds SQLite from source and needs none on the machine. Drop it to link the system SQLite and save about 1 MB; it also vendors OpenSSL when `native-tls` is on. The Nix builds still link the store's SQLite.

- **BREAKING**: every `--json` payload key is camelCase, and the four `read` commands answer `{"contents"}` rather than the printer's `{"message"}`. `caldav discover` renamed `calendar_home_set` to `calendarHomeSet` and `vdir list` `display_name` to `displayName`. The configuration vocabulary stays kebab-case, and every command's plain-text output is unchanged.
- **BREAKING**: item ids are the resource names a CalDAV server returned, verbatim, io-webdav no longer appending nor stripping `.ics`. That fixes read, update and delete addressing the wrong resource whenever an id did not end in `.ics`, and a create returning an unusable id when the server named the resource itself. Scripts pinning a hand-built id need updating.
- **BREAKING**: rewrote the wizard on the Himalaya model as `calendula configure` (alias `wizard`), replacing `account configure`. One prompt takes an email address, a server URL or a local folder path and its shape orients the rest; the account name is derived rather than prompted, and the account is tested before anything is emitted. It is offered where nothing can happen without a configuration, and skipped under `--json` or a non-terminal stdin. The generated account is written to a missing file, appended as plain text to an existing one so comments survive, or printed.
- **BREAKING**: dropped the io-calendar dependency and moved the cross-protocol layer into calendula, one adapter per protocol, following the cardamum precedent. io-calendar is frozen and pinned io-vdir 0.0.3 and io-webdav 0.0.1, so nothing below it could move while it stayed.
- **BREAKING**: renamed `completions` and `manuals` to `completion` and `manual`, and the shared subcommands to the singular `calendar`, `event` and `item`; the plural forms stay as hidden aliases. The `gcal` commands mirroring a Calendar API resource keep that API's spelling and gained hidden singular aliases.
- **BREAKING**: renamed the remote backend from `webdav` to `caldav` across the cargo feature, the subcommand and the config block. Only the io-webdav dependency keeps the WebDAV name.
- `create` and `update` take their iCalendar as an optional trailing positional (a path, raw contents, or `-` for stdin) on all four families, and bail when given neither a source nor `-i`. A create with no source mints its item, carrying a `PRODID`, a `UID`, a `DTSTAMP` and, on a VEVENT, a `DTSTART`; an update with no source starts from the stored item and sends the entity tag it read as `If-Match` unless `--if-match` names another. `item` names no component kind, so it mints nothing and says which families do.
- `create` and `update` answer a named output type rather than the printer's `Message`, an abandoned edit being an outcome they report. Both are untagged, so a write serializes as `{"id"}`.
- The calendar client selects its backend without connecting and opens on the first call that needs it, so a command running a composer holds none open while the editor is up: a server closes an idle connection, and the write after a long edit would read a dead socket. The four families resolve their account per subcommand, which is what lets `build` resolve none.
- `calendar create` reports the identifier the backend assigned rather than the one asked for. They differ only on Google, which mints its own.
- Made the parent calendar an optional `-k/--calendar` flag on every shared command instead of a positional, falling back to the new `calendar.default`. `calendar delete` takes it as a mandatory flag that never falls back.
- `caldav.server` takes a bare authority (`dav.example.org`, `domain:port`) as well as a full URL, defaulting to `https`, which is how the sibling products spell the same key.
- A credential command named by two backends of one account runs once per account instead of once per backend, so a `pass` or `gpg` entry shared by a `caldav` and a `gcal` block is unlocked once.
- A wizard-generated account no longer writes a `default = false` line.
- Adopted [ical-rs](https://github.com/pimalaya/ical) for iCalendar parsing, and an item whose bytes do not parse is skipped rather than failing the listing.
- Bumped the Pimalaya dependencies to their current releases: ical-rs 0.5, io-http 0.5, io-pimdir 0.5, io-vdir 0.1, io-webdav 0.3, pimalaya-cli 0.2, pimalaya-config 0.2, pimalaya-stream 0.3, and pimconf to its renamed successor io-pim-discovery 0.7. The `pimdir` feature is on by default.
- pimalaya-stream 0.3 retries a blocking stream reporting it is not ready for a minute before giving up, and arms a read deadline at connect time. **Behaviour change**: an exchange that used to end with a bare `Resource temporarily unavailable (os error 35)`, macOS especially, now survives it.
- Replaced the direct comfy-table dependency with pimalaya-cli's re-export, moving it to v8. `table.preset` keeps its v7 positional string, mapped onto the new typed style; the truncation indicator changes from `...` to `…`.
- Included the affected calendar or collection id in every `calendar`, `caldav` and `vdir` success message.
- Documented each command's JSON output shape as the last paragraph of its `--help`, replaced the root ARCHITECTURE.md with the src/main.rs header and the cairn specification, and slimmed the README Usage section to a pointer.
- Relicensed from AGPL-3.0-only to dual MIT OR Apache-2.0.

### Removed

- Removed `downloads-dir`, read by nothing: calendula downloads no attachment.

### Fixed

- Fixed CalDAV discovery settling on a URL with no DAV server behind it, such as a web page: every candidate must now answer a `PROPFIND`, and discovery fails otherwise rather than configure it (io-pim-discovery 0.8).

- A component listing narrows by kind before it paginates, so a page of `event list`, `todo list` or `journal list` holds that family. The page used to be taken over items of every kind and projected afterwards, so a calendar of thirty events and one todo answered `todo list` with an empty table. CalDAV pushes the kind down as an RFC 4791 `comp-filter` with the window nested inside it, gcal answers a non-VEVENT kind without a round-trip, and pimdir reads the kind off the stored summary when the body is not local.
- A composed VEVENT carrying no DTSTART is refused rather than sent. RFC 5545 3.6.1 requires it unless the calendar specifies a METHOD, which is a condition on the calendar that a per-component property list cannot state, and SabreDAV denormalizes DTSTART into its index on write and answers HTTP 500 rather than naming it.
- A source holding nothing but whitespace is refused, naming where it was read from, rather than handed to a backend as an empty body.
- `--config` reaches the wizard, which used to drop it on the one path where a user is most likely to pass it.
- `tls.cert` expands a `~` or an environment variable; `cert = "~/ca.pem"` used to look for a literal `./~/ca.pem`. Every configured path is now expanded when it is read rather than at each use.
- The pimdir backend lists the calendars it can take when refusing an unknown one, a collection id carrying a namespace that is not guessable.
- `completion` prints the script to stdout instead of writing files to the working directory, which broke every packaging helper capturing stdout. `manual` shares its shape: a positional list of command names and an optional `--dir`, so `calendula manuals ./man` becomes `calendula manual --dir ./man`.
- A CalDAV calendar's time zone is no longer silently dropped on create and update, io-webdav having read `calendar-timezone` when listing but never written it back.
- An item listing no longer keeps a calendar's own multistatus self-entry as a bogus item, which iCloud echoes.
- Restored the `Host` header on HTTP/1.1 requests (through io-http 0.2), whose absence made servers answer 400 and silently broke the `.well-known` discovery probe.
- `tls.cert` works for a self-signed server (through pimalaya-stream 0.1.1): the certificate is pinned to the server's leaf instead of registered as a trust anchor that rejected it.

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
