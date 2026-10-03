# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added the `wizard` cargo feature, on by default, gating the interactive configuration: the `configure` command and the offer a first run makes. A build without it drops the prompts and the dependencies only they use, and a missing configuration points at the documented sample instead.
- Added notes to the pimdir writes: a write passing on a source that supports it only in part ends with a `Note:` line, and a `notes` array under `--json`.
- Added recurrence expansion to `event list --from/--to`: every occurrence overlapping the window lists on its own, overrides applied and excluded dates dropped, with a `recurrenceId` addressing it and its `startsAt` and `endsAt` resolved to instants with their UTC offset.
- Added the attendees, the organizer, the description, the location, the status, the transparency and the online meeting link to the event JSON.
- Added `event read --recurrence-id`, projecting one occurrence, and `event find <UID>`.
- Added `pimdir reply` and `pimdir cancel`, queueing the `calendar-reply` and `calendar-cancel` intents the sync engine sends (pimdir STORAGE Annex B.2).

### Changed

- Changed the local date windows (vdir, msgraph, pimdir) to keep a recurring event whose occurrences fall in them, not only one starting there.
- Changed the pimdir writes to be refused before they are queued when a source of the store does not support them, naming the capability, the source and why (pimdir draft-03, STORAGE §15.6).
- Changed a scheduled event refused for want of a source that notifies to name the way through: marking its `ORGANIZER` and `ATTENDEE` with `SCHEDULE-AGENT=NONE`.

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
