# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added the first-run welcome: run without a configuration at a terminal, calendula prints on stderr the commands creating the file from the sample shipped beside the binary, or downloading it when none ships, then `event list`, and exits 1. Under `--json` or with stderr piped, the failure object or the error names the path looked at and the sample, now worded `No configuration found at <path>, start from the sample: <URL>`.
- Added config.sample.toml to the Nix package and the release archives as share/doc/calendula/config.sample.toml, which install.sh copies under its prefix and the first-run welcome copies from.
- Added `caldav.alpn`, the ALPN identifiers offered during the TLS handshake, `["http/1.1"]` when unset, an empty list skipping ALPN, as `gcal.alpn` and `msgraph.alpn` do.
- Added `nextPage` to the `--json` output of `event list`, `event find` and `item list`: the number of the page following the listing, as a string, `null` once it is complete (always on `event find`, which reads the calendar whole).
- Added `--page-token` to `event list` and `item list`, taking back the `nextPage` a listing printed, as cardamum and Himalaya do. A token that is not a page number is refused, naming it, and the token wins over a `-p` given beside it.
- Added `code` to each backend of the `account check --json` report, as Himalaya does: the stable code of its failure (`store-busy`, `store-uncreated`...), `null` when it passed or its failure carries none.
- Added `pimdir search <query>...` and `pimdir count <query>...`, querying the pimdir store's search index in the pimdir SEARCH language, read-only, over the account's events, every calendar unless `-k` names one. A hit renders with the `event list` columns, then its calendar, what matched and a snippet; `--sort date|name|relevance`, `--page` and `--page-size` page it. `date:` matches occurrences, a series included, its days, `today` and `yesterday` read in the local zone (`TZ`, then the system's). The answer carries its coverage, printed as one line under the table when incomplete, and a line names the index's zone when it reads floating times and dates in another.
- Added the error codes `index-missing`, `index-outdated`, `index-version`, `index-zone-unknown`, `index-fts5-missing`, `query-refused` and `query-unanswerable`; a refused query is drawn over its text in plain output.
- Added `--add-attendee [NAME:]ADDRESS` and `--remove-attendee ADDRESS` to `event build`, `event create` and `event update`, each repeatable: one attendee is added after the last one, invited plainly, or removed, and every other line keeps its bytes, the other attendees' answers included. Addresses compare case-insensitively, `mailto:` or not. An address already there, one the event lacks, one given to both flags, and either flag given with `--attendee` are refused before anything is written. ([#22](https://github.com/pimalaya/calendula/issues/22))
- Added `--send-updates all|external-only|none` to `event create`, `event update` and `event delete` on gcal accounts, sent as the API's `sendUpdates` so a write chooses whom Google mails: `none` fixes a typo or deletes an old event without mailing the guests. Left out, nothing changes. On `event create` it replaces the `all` an invitation the account organizes is inserted with; an event organized by someone else is imported, which mails nobody, so `all` and `external-only` are refused on it, naming the organizer. Every other backend refuses the flag by name. ([#21](https://github.com/pimalaya/calendula/issues/21))
- Added `--alarm <LEAD>` and `--no-alarm` to `event build`, `event create` and `event update`. `--alarm` replaces every VALARM of the event with one display alarm the lead before the start (an RFC 5545 duration such as `PT15M`, or `15m`, `1h`, `2d`), described by the event's summary; `--no-alarm` removes every VALARM. A lead the account cannot hold is refused before the write, never rounded: on gcal one that is not whole minutes or exceeds four weeks, on msgraph one that is not whole minutes or names days or weeks.
- Added `alarms` to the event JSON (`event list`, `event read`, `event find`, `event build`): every VALARM with its `action` and its `trigger` as written.
- Added `--online-meeting` to `event build`, `event create` and `event update`, writing `X-PIMDIR-ONLINE-MEETING:TRUE` (pimdir STORAGE Annex B.1) for the sync engine to create a Google Meet or Microsoft Teams meeting. The pimdir backend refuses it before queueing when no source that would push it declares `calendar.online-meeting` (one syncing the calendar for a create, one binding the event for an update); the other backends refuse the flag by name.
- Added field flags to `event build`, `event create` and `event update`, stacked between the source and `-i`: `--uid`, `--summary`, `--description`, `--location`, `--url`, `--start`, `--end`, `--duration`, `--time-zone`, `--status`, `--transparency`, `--organizer`, `--attendee`, `--categories` and `--sequence`, plus `--method` on `event build`. A flag replaces every instance of its property and leaves every other line as it was; a time takes a date, a local time, a `Z` time or an IANA zone prefix (`Europe/Paris:2026-10-19T09:00`), and a zone named arrives with its `VTIMEZONE`. `event update --start` alone moves the end with it. An event built from flags alone is checked; a source holding several VEVENTs is refused when a flag is set.
- Added the projection to `event build --json`: `{contents, method, events}`, the calendar's `METHOD` and its events as `event read` prints them, so a loose iCalendar (an invitation attached to a mail) is read with no account and no backend.
- Added `sequence` and `zoneAssumed` to the event JSON: the `SEQUENCE` (0 when absent), and whether a `TZID` nothing defines was read in the local zone.
- Added `default` to every calendar in `calendar list --json` (pimdir STORAGE §14): `true` on the one the server names the default, read on the pimdir backend from the role the sync engine recorded (`collections.role`), `false` on the other backends for now.
- Added recurrence expansion to `event list --from/--to`: every occurrence overlapping the window lists on its own, overrides applied and excluded dates dropped, with a `recurrenceId` addressing it and its `startsAt` and `endsAt` resolved to instants with their UTC offset.
- Added the attendees, the organizer, the description, the location, the status, the transparency and the online meeting link to the event JSON.
- Added `event read --recurrence-id`, projecting one occurrence, and `event find <UID>`.
- Added `pimdir reply` and `pimdir cancel`, queueing the `calendar-reply` and `calendar-cancel` intents the sync engine sends (pimdir STORAGE Annex B.2).
- Added `etag` to the event JSON (`event list`, `event read`, `event find`) and to `item read`, the version a write can be gated on. On pimdir it is the store's hash of the item's body.
- Added `--if-match` to `event delete` and `item delete` (CalDAV and gcal `If-Match`, msgraph `changeKey`, pimdir version; refused on vdir).
- Added `pimdir reply --recurrence-id` and `pimdir cancel --recurrence-id`, limiting the intent to one occurrence (`recurrence_id`, pimdir STORAGE Annex B.2); a performer unable to act on one occurrence is refused with `occurrence-unsupported`. Their JSON adds `recurrenceId`.
- Added a stable `code` to the `--json` error output for failures a caller acts on: `body-pending`, `precondition-failed`, `occurrence-unsupported`, `item-duplicate`, and the pimdir store states io-pimdir names `store-uncreated`, `store-version`, `store-foreign`, `store-corrupt`, `store-hash-algo`, `store-busy`, `store-exposed`, `flag-invalid` and `action-malformed`.
- Added the parked actions to `pimdir status`: every write the sync engine could not apply in the calendars shown, by its queue row (ROW), its calendar, its action and the error that parked it. Its JSON adds `parked`.

### Changed

- Changed `-c` to take one path per flag, repeated to merge several files (`-c public.toml -c private.toml`), never split on `:`; the `CALENDULA_CONFIG` variable is no longer read, `-c "$CALENDULA_CONFIG"` passing a path kept in a variable. **Breaking.**
- Changed the text output to plain text, with no border, no colour and nothing sized to the terminal. `event list` and `event find` print one line per event (id, start, end, summary, `-` for an empty one) between a header naming the account with its backend and the calendar (and the UID for `event find`) and a footer saying where the page stands (`Page: 1, events 1 to 25`, `, the last`, `empty`, `all` when unpaged) and how to reach the next (`Next page: --page 2`); `item list` the same with its entity tag, size and id, and `gcal instances` with its id, status, start, end and summary. `calendar list`, `caldav list`, `gcal calendars`, `gcal free-busy` and `pimdir search` print records the way `git log` prints commits; `account list`, `vdir list`, `gcal acl list` and `gcal colors` print aligned columns under an uppercase header; `account check`, `caldav discover`, `gcal settings` and `pimdir status` print aligned `Label: value` fields. `event agenda` no longer highlights today or the days holding an event, the events under the grid naming their days. Every string a server or a file gave is defused of control and bidi characters, and a refused pimdir query is drawn without colour. **Breaking.**
- Changed account resolution to pimalaya-config 0.4: several accounts marked `default = true` are refused, naming them, where one of them was taken at random; an unknown `-a <NAME>` lists the accounts the configuration holds; no default names both ways of picking one. **Breaking.**
- Changed config.sample.toml into one account listing the options every account shares with their values, then one commented section per backend or provider (CalDAV, Google Calendar, Google CalDAV, Microsoft Graph, Fastmail, vdir, pimdir), each a working account once uncommented, its OAuth 2.0 token read from `ortie token show`, and every variant of it parsed by the test suite. The pimdir root now names neverest's shared store, ~/.local/state/neverest, and the copied sample no longer holds an active CalDAV account pointed at caldav.example.org.
- Changed `-s 0` on `event list` and `item list` to list every item, where it listed none.
- Changed the clap help to plain, unwrapped text with the Pimalaya footer, and moved onto pimalaya-cli and pimalaya-config 0.4, pimalaya-cli taken from a git revision while its `records` module is unreleased; crossterm left the dependencies.
- Changed io-pim-discovery to 0.11: a DNS resolver that does not answer fails `caldav.discover`, naming it.
- Changed the pimdir backend to read one account of the store: with `pimdir.account` unset, the account every collection is grouped under, and a store several accounts share is refused, naming them, rather than read with their calendars merged. **Breaking.**
- Changed the pimdir write confirmations to say the write is queued for the next sync (`Event <id> queued for the next sync`, `... update queued ...`, `... deletion queued ...`) rather than created, updated or deleted. The JSON output is unchanged; a queued create's `id` is the link id it staged, which no command addresses before a sync applies it.
- Changed every write printing a confirmation to print a receipt instead of `{"message": ...}` under `--json`: `calendar create|update|delete` and `event|item delete` print `{id, queued}`, the item acted on and whether the write waits for a sync; `caldav create|delete`, `vdir create|rename|delete` and `gcal acl create|update|delete`, `gcal move` and `gcal quick-add` print `{id}`, a renamed collection's new id; `event|item build -o` prints `{path, size}`. The text form keeps its sentence, a queued delete now naming the item (`Event <id> deletion queued for the next sync`). Every command now has a schema under `json-schema` (pimalaya-cli 0.3). **Breaking.**
- Bumped pimalaya-cli and pimalaya-config to 0.3, and io-pim-discovery to 0.9. The `--json` failure object always carries `code`, `null` when none applies.
- Changed the pimdir writes to refuse a to-do (`item` with a VTODO) into a calendar whose source states it takes none (`calendar.component.vtodo` `none` on the calendar or source-wide, pimdir STORAGE §15.6), before anything is queued, naming the source and pointing at `-k/--calendar`. A source stating nothing of it refuses no to-do.
- Bumped rustls-webpki to 0.103.16 and webpki-root-certs to 1.0.11, fixing RUSTSEC-2026-0382 and RUSTSEC-2026-0383.
- Changed piped `--json` output to print one value per line, each schema of `json-schema` to carry an `$id` naming the contract version (`v0.2`), and added `calendula-error`, the schema of the JSON failure object (pimalaya-cli `55b6693`). The abandoned arm of the `create` and `update` outputs, which `--json` never prints, is now `{}` rather than `null`, so each of their schemas is an object.
- Changed the `event list` table to show the start and end in the local zone (`2026-10-30 09:00`) rather than as iCalendar spells them, a whole-day event by its days with the end as its last one, and kept whole when the table is squeezed; the `--json` output is unchanged. `event list --help` now points to `--from` or `event agenda` for what is coming, the listing without a window being the raw one.
- Changed `event create` and `event update` on a Google or an Outlook account to read a `--start` or `--end` naming no zone in the local zone (`TZ`, then the system's), as `--time-zone` would, instead of writing a floating time the backend refused. `--time-zone` still wins; the other backends keep the time floating.
- Changed a Google account to accept an event giving its end as a `DURATION` (`event create --duration`, an invitation) instead of refusing it: the end is computed from the `DTSTART`, in its zone, and the event is stored with a `DTEND` (io-gcal 0.3). A negative `DURATION`, or one with hours after a date start, is still refused.
- Changed `event agenda` to name months and weekdays in the time locale, as cal does: `LC_ALL`, then `LC_TIME`, then `LANG`, with the English names of the POSIX locale when none is set or chrono does not know it. The event times under the grid are stamped in it too, and a month argument is read in English or in that language.
- Changed the pimdir listings to stream the store rather than load it: `event list`, `event find`, `item list` and `pimdir status` hold one batch of rows and the page they print instead of every body of the calendar, a page stops reading once full, and a body is not read where the stored summary already rules the item out, by its kind or, for `event find`, by its `UID`. The output is unchanged.
- Changed an empty value given to `--summary`, `--description`, `--location`, `--url`, `--categories`, `--organizer` or `--attendee` to remove the property rather than write it empty (`--attendee ""` removes every attendee). The time flags, `--status`, `--transparency` and `--sequence` still refuse an empty value.
- Changed `event update` to revise the event it writes: `DTSTAMP` is set to the time of the write in UTC, and `LAST-MODIFIED` to the same instant when the event carries one, whether the write came from field flags, a source or `-i`.
- Changed the composer from the only authoring surface of an event to the complete one: the field flags cover the common fields, and a recurrence, an alarm, a conference or a hand-written `VTIMEZONE` stay the composer's. The "nothing to build", "nothing to create" and "nothing to update" errors of the `event` family name the field flags.
- Changed `onlineMeetingUrl` to prefer, among the `CONFERENCE` properties of an event (one per way in, RFC 7986 5.11), the one whose `FEATURE` says `VIDEO`, then a web one, so a dial-in `tel:` is never taken for the link; the vendors' own properties come next.
- Changed io-pimdir to 0.10, so calendula shares one io-pimdir and one store schema with the rest of the set: a reply or cancellation the sync engine performed now leaves a receipt, a store reconciled by the current neverest (no `probes` table) opens, an empty mail date is stored as no date, a band round keeps undated mail, and opening a store creates the `items_by_sort_global` index a page across collections walks.
- Changed ical-rs to 0.6, io-gcal to 0.2 and io-msgraph to 0.5. The composed-item check now also refuses a duration outside RFC 5545 3.3.6 (a `TRIGGER:P1H`, say) and a `LANGUAGE` or `ALTREP` on a property the RFC gives none, and `--duration` refuses a second straight after an hour (`PT1H5S`; write `PT1H0M5S`). The gcal and msgraph backends mint an online meeting's join link as a `CONFERENCE` (RFC 7986) rather than their vendor property.
- Changed ical-rs to 0.7, io-gcal to 0.3.1, io-msgraph to 0.7.1, io-pimdir to 0.10 and pimalaya-cli to 0.3.2, every Pimalaya dependency taken from crates.io: calendula links one ical-rs and one io-http where it linked two.
- Changed io-msgraph to 0.6 and io-http to 0.6 under the REST backends. A response whose head passes 64 KiB or whose body passes 256 MiB now fails the exchange with a typed error instead of growing the buffer without bound, a connection closing mid-body fails rather than spinning, an iCalendar body nests 16 components at most, and the pimdir backend reads a file item of the store (Annex A.7) as dateless.
- Bumped io-webdav to 0.7, and io-http to 0.6 under the CalDAV backend. A calendar outside the home set is listed and addressed by its path (Google's `/caldav/v2/<calendar>/events/`) rather than its last segment, a request the server closed the connection on is resent once, and a CalDAV multistatus is parsed 3.4 to 9 times faster, without quadratic copies and nested 64 levels at most.
- Changed the pimdir `update_item` and `delete_item` to honour `--if-match`: a write naming a version the item no longer has is refused before anything is queued, with an error starting `Precondition failed:`.
- Changed the local date windows (vdir, msgraph, pimdir) to keep a recurring event whose occurrences fall in them, not only one starting there.
- Changed the refusal of a pimdir store calendula cannot open to say what to run or upgrade rather than to delete the store and let it resync: a store an earlier sync engine wrote is reinstalled by it (`neverest sync --reset --force -a <account>` once nothing is queued, which moves it aside), and one a newer engine wrote needs a newer calendula.
- Changed the pimdir writes to be refused before they are queued when a source of the store does not support them, naming the capability, the source and why (pimdir STORAGE §15.6). A write a source supports only in part logs a warning.
- Changed a scheduled event refused for want of a source that notifies to name the way through: marking its `ORGANIZER` and `ATTENDEE` with `SCHEDULE-AGENT=NONE`.
- Changed `account check` on pimdir to probe a producer after opening the store, so a store a write cannot reach fails the check rather than the first write, with `store-busy` at once while a collector holds its staging lock; the check writes nothing into the store.
- Changed the pimdir backend to follow pimdir storage-draft-09, sync-draft-08 and search-draft-09 (io-pimdir 5c18a02). A store any earlier neverest wrote is refused as `store-version`, never migrated: sync it with that neverest until nothing is queued, upgrade, then `neverest sync --reset --force -a <account>` moves it aside and syncs a new one. `pimdir reply` and `pimdir cancel` now always name the source performing them, which the payload requires: in a calendar whose sources declare nothing, `--source` or else the one source syncing it, several being refused, so their JSON `source` is never `null`. An object whose blob entry is no body (a directory, a link, bytes not hashing to its name) now lists with a warning, as a missing one does. **Breaking.**
- Changed a CalDAV `server` or `home` to take an ASCII host only, an internationalised one written in Punycode (`https://xn--bcher-kva.example/` for `bücher.example`): a non-ASCII host is refused when parsed, `home` when the configuration loads and `server` when the account connects, rather than converted. **Breaking.**
- Changed the base64 and quoted-printable codecs linked through the dependencies (the CalDAV `Basic` header io-http encodes among them) to use the SIMD features of the CPU, detected at startup, rather than the portable code. The output is unchanged.

### Removed

- Removed `-w/--max-width` from `calendar list`, `event list`, `event find`, `item list` and `pimdir search`. **Breaking.**
- Removed the table and colour options: `table.preset`, `table.arrangement`, the `*-color` keys of `calendar.list.table`, `event.list.table`, `item.list.table` and `account.list.table`. A `table` or top-level `account` table left in a configuration is refused by name, and a colour key is refused by the block that held it; the `page-size` keys stay. **Breaking.**
- Removed the configuration wizard: the `configure` command (alias `wizard`), the offer a bare `calendula` and a command without a configuration made, the `wizard` cargo feature and the `calendula-configure` JSON Schema. Write the configuration from config.sample.toml; a command finding none fails naming the path it looked at and the sample, and a bare `calendula` prints the help. io-pim-discovery now comes with the `caldav` feature alone, for `caldav.discover`. **Breaking.**
- Removed the `todo` and `journal` commands (`list`, `read`, `build`, `create`, `update`, `delete`) and their `todo.list` and `journal.list` options: calendula is the events CLI, to-dos moving to [unfog](https://github.com/pimalaya/unfog) and journal entries to [journal](https://github.com/pimalaya/journal). `item` still reads and writes any component raw. A `todo` or `journal` table left in a configuration, at the top level or under an account, is refused by name, pointing at the CLI that took it over. `item build -i` and `item create -i` given no source now name `calendula event`, `unfog` and `journal` as what composes an item. **Breaking.**

### Fixed

- Fixed `event read` and `item read` printing the stored iCalendar's control characters as is, which could drive the terminal: the text view replaces them and the bidi controls, keeping tabs and line feeds and reading CRLF as LF, and `--json` keeps the stored text.
- Fixed the lines under `pimdir search` and `pimdir count` printing the index zone and the unsynced days as the store holds them.
- Fixed a credential command run again each time a unified command reopened its backend (after a composer, say): the account's secrets now go through one resolver for the run, so a command is spawned once per account.
- Fixed `event agenda` writing terminal escapes into its grid even when piped.
- Fixed a Google account refusing an event with neither `DTEND` nor `DURATION`: it is written as ending at its start, or the next day after a date start (RFC 5545 3.6.1), and no `DTEND` equal to a timed `DTSTART` is read back (io-gcal 0.3.1).
- Fixed an Outlook event's alarm that Graph's reminder cannot hold coming back with its lines scattered across the event after a write (io-msgraph 0.7.1).
- Fixed a CalDAV `server` written as a URL that fails to parse (an out-of-range port, a space in the host) being read as a bare authority, `https://` prefixed, and so sent to host `https`: it is refused.
- Fixed `event create --attendee` on a Google account mailing no invitation: an event naming attendees and no organizer, or the account as its organizer, is now inserted with `sendUpdates=all` rather than imported, keeping its `UID`. An event organized by another address is still imported, mailing nobody. ([#20](https://github.com/pimalaya/calendula/issues/20))
- Fixed a pimdir create or update losing its body: the body was written before the producer took the store's staging lock, so a collector running in between deleted it and the sync engine applied an event with no contents. The body is now staged through the producer that enqueues it, under its lock.
- Fixed a pimdir create whose `UID` the calendar already holds, or has queued, reported as created then parked by the sync engine: it is refused with `item-duplicate` before anything is queued.
- Fixed a pimdir create or update the store refused (a duplicate `UID`, a to-do into a calendar taking none, a scheduled event no source notifies for) leaving its body in the store's blob tree until a collection: the body is now written only with the queued action that pins it.
- Fixed a pimdir update made over the version it read (`event` or `item update` reading the stored item, or given `--if-match`) overwriting a change a sync landed while it waited in the queue: the queued action names that version, and the sync engine parks the update, shown by `pimdir status`, rather than applying it over the other change.
- Fixed `pimdir reply` and `pimdir cancel` queueing their intent unchecked when the event's body blob is missing from the store: they now fail, as `event read` does.
- Fixed `account check` exiting 0 when a backend failed: it now prints its report, in text and as the `--json` result, and exits 2, as cardamum and Himalaya do.
- Fixed `event agenda` reading a lone month name as a year: `event agenda noviembre` drew the whole current year, where cal draws that month. It now draws that month of the current year, a lone number still being a year. A day that is not a number or not in the month, a month matching no name, a month number outside 1-12 or an unreadable year is refused, naming the value, instead of drawing the 1st, January or the current year.
- Fixed `event agenda` placing a recurring series once, at its first start: a yearly event created years ago marked none of the days on screen. The agenda now reads the days its grid draws and expands each series over them, as `event list --from/--to` does.
- Fixed the composed-item check refusing a whole-day, a zoned or an organized event: a `DATE` value and a `TZID` on `DTSTART`, `DTEND`, `DUE`, `RECURRENCE-ID`, `EXDATE` and `RDATE`, and a `CN` on `ORGANIZER`, all of which RFC 5545 allows.
- Fixed a `TZID` with a leading slash (`/Europe/Paris`) read in the local zone rather than its own.
- Fixed a `VTIMEZONE` stating no observance read as UTC: it defines nothing, and the time is read in the local zone with `zoneAssumed`.
- Fixed an `EXDATE` or a `RECURRENCE-ID` written in UTC (or another zone) on a zoned recurring series naming no instance: the excluded occurrence still listed, and the override listed beside the occurrence it replaced. Each is now told on the series' clock first, as a UTC `UNTIL` already was.
- Fixed `event delete` taking only the backend id although its help named the iCalendar `UID`: it now takes either, the id first, and refuses a `UID` several events carry.
- Fixed a msgraph listing failing on a `numbered` recurring series: Graph fills its `endDate` with `0001-01-01`, which bounded the instances request before its start. The window now follows the range type. ([#10](https://github.com/pimalaya/calendula/issues/10))
- Fixed msgraph listings dropping the cancelled occurrences of a series: Graph returns them only on a read of the master by id, so each master is now read so before projection. ([#11](https://github.com/pimalaya/calendula/issues/11))
- Fixed msgraph listings repeating an event Graph returned on both sides of a page boundary; one id at two revisions is now an error. ([#12](https://github.com/pimalaya/calendula/issues/12))

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
