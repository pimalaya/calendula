# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added a shared command family per iCalendar component kind: `todo` (VTODO) and `journal` (VJOURNAL) join `event` (VEVENT), each with `list`, `read`, `create`, `update` and `delete`.

  A todo listing renders the summary, due date, status, priority and completion percentage a task list is read by; a journal listing renders the summary, date and status of a dated note. They are views over the same items, so they add no backend operation, and `item` stays the raw unfiltered one. `--from` / `--to` narrow them, applied after parsing rather than pushed down: a server-side range filter is defined against a component's start and end, which a todo and a journal entry do not both carry. VFREEBUSY and VTIMEZONE get no family, the first being the answer to a query and the second the definition of the zones the others reference.

- Added the `gcal` command family, covering the half of the Calendar API that iCalendar cannot express: `calendars` (the richer listing, with the access role, the primary flag, the time zone and the default reminders), `acl` (`list`, `create`, `update`, `delete`), `free-busy`, `instances`, `move`, `quick-add`, `colors` and `settings`.

  Push channels are deliberately absent: a channel delivers to an HTTPS endpoint the caller must host, which a CLI has not. So are `calendars.clear` and `transferOwnership`, both irreversible.

- Added the `gcal` backend: calendula over the Google Calendar API v3, through [io-gcal](https://github.com/pimalaya/io-gcal).

  Google is reachable over CalDAV only in a crippled form (bearer tokens only, `MKCALENDAR` refused, an off-spec discovery entry point), so a Google account was read-mostly and calendar creation impossible. The native backend sits behind a `gcal` cargo feature and an `[accounts.<name>.gcal]` block carrying a bearer `auth.token` secret, which an OAuth 2.0 token broker fills like any other command. Calendars can now be created, updated and deleted, a date range is pushed down as `timeMin` / `timeMax`, a listing walks `nextPageToken` only as far as the requested page, and an update honours `if_match` through `If-Match`.

  Google stores a JSON event and exposes no per-event iCalendar representation, so the backend synthesizes the document of record and re-projects it on write: fields with a well-defined iCalendar slot are managed both ways, Google-only fields survive an update untouched, provider-scoped ones are minted as read-only `X-GOOGLE-*` properties, and every remaining line is stashed verbatim in `extendedProperties.private`. Only VEVENT projects: a VTODO or VJOURNAL is refused by name, since Google models neither.

  Google returns a boundary as an absolute instant plus the calendar's display zone, so only the offset decides the instant and the zone is carried over from the server copy on update. That keeps a zoned recurring series expanding where it did, rather than falling back to UTC and drifting by an hour after a daylight-saving change.

  A boundary anchored in a named zone carries the VTIMEZONE it references, minted from the zone name Google sends, so an item stands on its own as an .ics file. The Calendar API carries the IANA name and nothing behind it, so the `gcal` feature carries a time zone database of its own.

- `calendar create` now reports the identifier the backend assigned rather than the one asked for. They differ only on Google, which mints its own, and the reported id is the one later commands address the calendar by.

- Added the `pimdir` backend: calendula over a local [pimdir](https://github.com/pimalaya/pimdir) store, the offline cache a sync engine fills.

  It sits behind a `pimdir` cargo feature and a `[accounts.<name>.pimdir]` block carrying a `root` and an optional `source`. Reads are availability-aware: an item the sync listed but has not downloaded still shows in a listing, and reading it reports "body not fetched" rather than failing. Writes are staged io-replica mutations the next sync pushes, attributed to `source` and refused outright on a store never synced as it. Calendars come from the sync, so `calendar create`, `update` and `delete` refuse here.

- Added `pimdir status`, reporting the source writes are attributed to, every source the store has been synced as, and how many of each calendar's items carry a local body.

- Added `--from` / `--to` date-range filtering to `event list` (YYYY-MM-DD, both inclusive). CalDAV pushes it server-side as an RFC 4791 `time-range` filter; the local backends apply it after parsing, and pimdir answers it from the stored summary when the body is not local. A range also lifts the default page-size cap, so every match is returned.

- Added the `-b/--backend` flag selecting which backend the shared commands target. The default, `auto`, takes the first configured one in calendula's priority order (vdir, pimdir, caldav, gcal).

- Adopted the [Cairn](https://github.com/pimalaya/cairn) convention: cairn/spec holds the living specification (backends, commands, config, wizard, packaging), cairn/changes the proposals, cairn/log the dated history.

### Changed

- **BREAKING** Rewrote the wizard on the Himalaya model, and made bare `calendula` run it.

  One prompt now takes an email address, a server URL or a local folder path, and its shape orients the rest: an address runs bounded parallel discovery and each reachable server becomes one entry, a URL is taken as the CalDAV context root, a folder is detected as a vdir home or a pimdir store. The account name is derived from the input rather than prompted, the account is tested before anything is emitted, and the result is printed as a TOML document on stdout, saved to a file only when stdout is a terminal and you ask. It no longer writes to disk on its own, and no longer runs implicitly when a command finds no configuration.

- **BREAKING** Removed `account configure`. It wrote to disk, which the new printing wizard does not, and Himalaya has no equivalent. Run `calendula` to generate an account and merge it into your configuration.

- **BREAKING** Dropped the io-calendar dependency and moved the cross-protocol layer into calendula, following the cardamum precedent: the shared types and the backend dispatcher are the product's own, with one adapter per protocol. io-calendar is frozen and still pinned io-vdir 0.0.3 and io-webdav 0.0.1, so nothing below it could move while it stayed.

- **BREAKING** Item ids are now the resource names a CalDAV server returned, verbatim.

  io-webdav no longer appends nor strips a `.ics` extension, which fixes item read, update and delete addressing the wrong resource whenever an id did not end in `.ics`, and a create returning an unusable id when the server named the resource itself. An id a listing shows now round-trips through every verb. Scripts pinning a hand-built id need updating.

- Adopted [ical-rs](https://github.com/pimalaya/ical) for iCalendar parsing. io-calendar's `as_ical` went with the crate and io-vdir 0.1 removed its own parser, so the VEVENT projection behind `event list` and `event agenda` now lives in one place. An item whose bytes do not parse is skipped rather than failing the whole listing.

- Bumped every remaining Pimalaya dependency to its current release: io-vdir 0.1, io-http 0.3, pimalaya-cli 0.2, pimalaya-config 0.1.1, pimalaya-stream 0.1.2, and pimconf to its renamed successor io-pim-discovery 0.5.

- Replaced the direct comfy-table dependency with pimalaya-cli's re-export, which moves it to v8. The `table.preset` option keeps its v7 positional string, mapped onto the new typed style, so existing configurations stay valid; the default truncation indicator changes from `...` to `…`.

- Replaced the root ARCHITECTURE.md with the `src/main.rs` header and the cairn specification, following the org convention that retired the per-repo architecture document.

- Documented each command's JSON output shape as the last paragraph of its `--help` text, and slimmed the README Usage section down to a pointer to `calendula --help`.

- Extracted the `-k/--calendar CALENDAR-ID` flag into a shared argument reused across the whole shared API. `calendar update` takes it (replacing its positional id, with the usual `calendar.default` fallback) and `calendar delete` takes it as a mandatory flag that never falls back.

- Made the parent calendar of every `event` and `item` command an optional `-k/--calendar CALENDAR-ID` flag instead of a positional argument; when omitted it falls back to the new `calendar.default` config, otherwise the command bails.

- Changed `event create`/`event update` and `item create`/`item update` to take their iCalendar as a trailing positional `ICAL` argument (a path, raw iCalendar contents, or `-` for stdin) instead of reading only from stdin, matching `tcal edit`.

- Included the affected calendar or collection id in every `calendar`, `caldav` and `vdir` success message.

- Migrated to the pimalaya-cli / pimalaya-config / pimalaya-stream stack and adopted the Himalaya v2 CLI structure.

- Renamed the shared subcommands to the singular `calendar`, `event` and `item` to match Himalaya; the plural forms stay as hidden aliases.

- Renamed the remote backend from `webdav` to `caldav` across the public surface: the cargo feature, the subcommand and the config block. Only the underlying io-webdav dependency keeps the WebDAV name.

- Relicensed from AGPL-3.0-only to dual MIT OR Apache-2.0.

### Fixed

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
