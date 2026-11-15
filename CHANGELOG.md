# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added the WebDAV and CalDAV methods to `caldav`, as cardamum's `carddav` has them: `propfind` (the calendars with their CTag, sync token and components, or one calendar's items by id and ETag), `proppatch` (`-n`, `-d`, `-C`, `--tz`), `mkcalendar`, `report query` (a `calendar-query` built from `--component`, `--from`, `--to`, `--match`, `--not-match`, `--defined`, `--not-defined` and `--collation`), `report multiget`, `report sync`, `report raw`, `get` and `put`, and `delete` of an item as well as of a calendar. `caldav list` (and `ls`) and `caldav create` stay hidden aliases of `propfind` and `mkcalendar`, printing what they printed, `--display-name` read as `-n`; `discover` adds `server` and reads an endpoint it cannot resolve as `null` rather than failing.
- Added `msgraph`, the Microsoft Graph namespace, as cardamum's: `calendars list`, `get`, `create`, `rename` and `delete`, `events list`, `get`, `create`, `update`, `delete` and `delta` (a calendar view delta over `--from` and `--to`, its next round opened through `--delta-link`), `profile get` and `request`, each printing Graph's own JSON, `request` refusing a full URL off Graph's origin before anything is sent so the token never leaves for another host. The listings print `nextPage`, Graph's `@odata.nextLink` lifted out of the page, `null` on the last one, and take it back through `--page-token`, as every Pimalaya program pages a protocol listing. An event or a calendar Graph does not hold fails `item-not-found` or `calendar-not-found`.
- Added `gcal request <METHOD> <PATH> [JSON]`, as cardamum's `gpeople request` and calendula's `msgraph request`: any Calendar API request, the path joined onto the `v3/` base or a full URL on Google's API origin (another origin refused before anything is sent, so the token never leaves for another host), an optional JSON body, printing `{response}`, the JSON Google answered.
- Added `vdir item list`, `get`, `create`, `update`, `delete`, `copy` and `move` (alias `items`), the raw files of a collection of any kind, as cardamum's.
- Added `repl`, a session running the CLI's commands over one account, the unified client connecting once, as in Himalaya and cardamum. A line takes `--json` for itself and a `#` comment; another session, the composer (`-i`), stdin input and the flags the session fixes (`-a`, `-c`, `-b`, `--log-level`) are refused by name, the session going on.
- Added `mcp`, a Model Context Protocol server exposing the unified API as tools: `account_capability`, `calendar_list`, `event_list`, `event_read`, `event_find`, `item_list` and `item_read`; the calendar, event and item create, update and delete tools under `--allow-write`; the scheduling arguments (`attendee`, `add-attendee`, `remove-attendee`, `organizer`, `send-updates`) under `--allow-send` as well, which requires `--allow-write`. Without `--allow-send`, a write or a delete of an item naming an `ATTENDEE`, in the iCalendar given or in the one stored, is refused, since the server or a sync may notify the attendees. A tool the account's backend does not serve is not listed, and no tool reads a local path. `openWorldHint` marks the tools listing an argument that invites or notifies people, `event_add`, `event_update` and `event_delete` under `--allow-send`, a gate moving no hint by itself.
- Added `account capability`, listing what an account serves of the unified API without connecting: one row per capability, `{name, backend, level, detail}` under `--json`, the backend it resolves to (`null` when the account configures none `--backend` allows), its level (`full`, `partial`, `server`, `none`) and what the level leaves out; `--backend <name>` reads another backend of the account. Its output is pimalaya-cli's, shared with Himalaya and cardamum. The unified API is now the union of what the backends serve, as in Himalaya and cardamum, and each unified command's `--help` ends with a `Backends:` footer naming the backends serving what it reaches. A verb or a flag the account's backend does not serve is refused with `capability-missing` before anything is read or connected, the message naming the backend, the capability (`item.remove.if-match`, `online-meeting`, `scheduling.send-updates`, `calendar.description`...), why, and the backend of the account serving it, if any.
- Added a stable `code` to the `--json` failure object, `null` when none applies, for the failures a caller acts on: `capability-missing`, `stdin-taken` (a command reading an iCalendar from stdin inside a session), `body-pending`, `precondition-failed`, `occurrence-unsupported`, `item-duplicate`, the pimdir store states io-pimdir names (`store-uncreated`, `store-version`, `store-foreign`, `store-corrupt`, `store-hash-algo`, `store-busy`, `store-exposed`, `lock-replaced`, `flag-invalid`, `action-malformed`) and the search index ones (`index-missing`, `index-outdated`, `index-version`, `index-zone-unknown`, `index-fts5-missing`, `query-refused`, `query-unanswerable`).
- Added the first-run welcome: run without a configuration at a terminal, calendula prints on stderr the commands creating the file from the sample shipped beside the binary, or downloading it when none ships, then `event list`, and exits 1. Under `--json` or with stderr piped, the failure object or the error names the path looked at and the sample (`No configuration found at <path>, start from the sample: <URL>`).
- Added config.sample.toml to the Nix package and the release archives as share/doc/calendula/config.sample.toml, which install.sh copies under its prefix and the first-run welcome copies from.
- Added `caldav.alpn`, the ALPN identifiers offered during the TLS handshake, `["http/1.1"]` when unset, an empty list skipping ALPN, as `gcal.alpn` and `msgraph.alpn` do.
- Added `nextPage` to the `--json` output of `event list`, `event find` and `item list`, the number of the page following the listing as a string, `null` once it is complete (always on `event find`, which reads the calendar whole), and `--page-token` to `event list` and `item list`, taking it back, as cardamum and Himalaya do. A token that is not a page number is refused, naming it, and the token wins over a `-p` given beside it.
- Added `code` to each backend of the `account check --json` report, as Himalaya does: the stable code of its failure (`store-busy`, `store-uncreated`...), `null` when it passed or its failure carries none.
- Added `pimdir search <query>...` and `pimdir count <query>...`, querying the pimdir store's search index in the pimdir SEARCH language, read-only, over the account's events, every calendar unless `-k` names one. A hit prints as a record, its summary then its snippet, its JSON carrying the event projection, its calendar and what matched; `--sort date|name|relevance`, `--page` and `--page-size` page it. `date:` matches occurrences, a series included, its days, `today` and `yesterday` read in the local zone (`TZ`, then the system's). The answer carries its coverage (pimdir SEARCH §8.2: `complete`, `bodiless`, `missing`, `unparseable`, `unindexed`, `behind`, `since`, `until`, `filling`, `horizonLeft`, `cut`, `zoneDiffers`), printed as one line under the hits when incomplete: an index behind the store is, and so is an event not downloaded, or a body the index could not read or parse, while a term of the query reads a calendar body (a text term, `date:`), so an event listed and not downloaded leaves `organizer:alice@example.org` complete. A line names the index's zone when a `date:` term reads its days in another than the index read floating times and dates in, which leaves the answer complete, and a refused query is drawn over its text.
- Added `--add-attendee [NAME:]ADDRESS` and `--remove-attendee ADDRESS` to `event build`, `event create` and `event update`, each repeatable: one attendee is added after the last one, invited plainly, or removed, and every other line keeps its bytes, the other attendees' answers included. Addresses compare case-insensitively, `mailto:` or not. An address already there, one the event lacks, one given to both flags, and either flag given with `--attendee` are refused before anything is written. ([#22](https://github.com/pimalaya/calendula/issues/22))
- Added `--send-updates all|external-only|none` to `event create`, `event update` and `event delete` on gcal accounts, sent as the API's `sendUpdates` so a write chooses whom Google mails: `none` fixes a typo or deletes an old event without mailing the guests. Left out, nothing changes. On `event create` it replaces the `all` an invitation the account organizes is inserted with; an event organized by someone else is imported, which mails nobody, so `all` and `external-only` are refused on it, naming the organizer. Every other backend refuses the flag with `capability-missing`. ([#21](https://github.com/pimalaya/calendula/issues/21))
- Added `--alarm <LEAD>` and `--no-alarm` to `event build`, `event create` and `event update`. `--alarm` replaces every VALARM of the event with one display alarm the lead before the start (an RFC 5545 duration such as `PT15M`, or `15m`, `1h`, `2d`), described by the event's summary; `--no-alarm` removes every VALARM. A lead the account cannot hold is refused before the write, never rounded: on gcal one that is not whole minutes or exceeds four weeks, on msgraph one that is not whole minutes or names days or weeks.
- Added `alarms` to the event JSON (`event list`, `event read`, `event find`, `event build`): every VALARM with its `action` and its `trigger` as written.
- Added `--online-meeting` to `event build`, `event create` and `event update`, writing `X-PIMDIR-ONLINE-MEETING:TRUE` (pimdir STORAGE Annex B.1) for the sync engine to create a Google Meet or Microsoft Teams meeting. The pimdir backend refuses it before queueing when no source that would push it declares `calendar.online-meeting` (one syncing the calendar for a create, one binding the event for an update); the other backends refuse the flag with `capability-missing`.
- Added field flags to `event build`, `event create` and `event update`, stacked between the source and `-i`: `--uid`, `--summary`, `--description`, `--location`, `--url`, `--start`, `--end`, `--duration`, `--time-zone`, `--status`, `--transparency`, `--organizer`, `--attendee`, `--categories` and `--sequence`, plus `--method` on `event build`. A flag replaces every instance of its property and leaves every other line as it was; a time takes a date, a local time, a `Z` time or an IANA zone prefix (`Europe/Paris:2026-10-19T09:00`), and a zone named arrives with its `VTIMEZONE`. `event update --start` alone moves the end with it. On a Google or an Outlook account, which cannot store a floating time, a `--start` or `--end` naming no zone is read in the local zone (`TZ`, then the system's) unless `--time-zone` names one. An empty `--summary`, `--description`, `--location`, `--url`, `--categories`, `--organizer` or `--attendee` removes its property, and `--duration` refuses a second straight after an hour (`PT1H5S`; write `PT1H0M5S`). An event built from flags alone is checked; a source holding several VEVENTs is refused when a flag is set. The composer stays the complete surface, for a recurrence, another alarm, a conference or a hand-written `VTIMEZONE`, and the "nothing to build", "nothing to create" and "nothing to update" errors of the `event` family name the flags.
- Added the projection to `event build --json`: `{contents, encoding, method, events}`, the calendar's `METHOD` and its events as `event read` prints them, so a loose iCalendar (an invitation attached to a mail) is read with no account and no backend.
- Added `sequence` and `zoneAssumed` to the event JSON: the `SEQUENCE` (0 when absent), and whether a `TZID` nothing defines was read in the local zone.
- Added `default` to every calendar in `calendar list --json` (pimdir STORAGE §14): `true` on the one the server names the default, read on the pimdir backend from the role the sync engine recorded (`collections.role`), on Google the primary calendar, on Graph the one it marks default (`isDefaultCalendar`), `false` on CalDAV and vdir.
- Added recurrence expansion to `event list --from/--to`: every occurrence overlapping the window lists on its own, overrides applied and excluded dates dropped, with a `recurrenceId` addressing it and its `start` and `end` resolved to instants with their UTC offset, `Z` for UTC as Himalaya spells it.
- Added the attendees, the organizer, the description, the location, the status, the transparency and the online meeting link to the event JSON. `onlineMeetingUrl` prefers, among the `CONFERENCE` properties of an event (one per way in, RFC 7986 5.11), the one whose `FEATURE` says `VIDEO`, then a web one, so a dial-in `tel:` is never taken for the link; the vendors' own properties come next.
- Added `event read --recurrence-id`, projecting one occurrence, and `event find <UID>`.
- Added `pimdir reply` and `pimdir cancel`, queueing the `calendar-reply` and `calendar-cancel` intents the sync engine sends (pimdir STORAGE Annex B.2), and failing as `event read` does when the event's body is missing from the store. Each names the source performing it, so their JSON `source` is never `null`: `--source`, else in a calendar whose sources declare nothing the one source syncing it, several being refused. `--recurrence-id` limits the intent to one occurrence, a performer unable to act on one being refused with `occurrence-unsupported`. Each prints `{row, id, calendarId, recurrenceId, partstat, comment, source}`, `row` the queue row `pimdir queue show` and `pimdir queue cancel` take, `id` the event.
- Added `etag` to the event JSON (`event list`, `event read`, `event find`) and to `item read`, the version a write can be gated on. On pimdir it is the store's hash of the item's body.
- Added `--if-match` to `event delete` and `item delete` (CalDAV and gcal `If-Match`, msgraph `changeKey`, pimdir version; refused on vdir with `capability-missing`).
- Added the parked actions to `pimdir status`: every write the sync engine could not apply in the calendars shown, by its queue row (ROW), its calendar, its kind and the error that parked it. Its JSON adds `parked`, each row `{row, calendarId, kind, error}`, `kind` kebab-case as `pimdir queue list` and Himalaya spell it. An update or a delete over an event a sync left conflicted is parked there, calendula resolving no conflict. It adds `refused` too, each event a server refused for good `{calendarId, itemId, source, reason, refusedAt}`, held by the sync engine until the event is edited or removed.
- Added `errors` to `calendar list`, `event list`, `event find`, `event agenda` and `item list`, as Himalaya does: one item error `{id, code, error, sources}` per calendar or item the listing could not read, `[]` when there is none, `id` `null` when the backend named none (a CalDAV listing the server truncated). The text output prints one `Error:` line each, `<id>  <code>  <error>`, last, after the output and one blank line, nothing when there is none. The `calendula-item-error` schema describes it, and the MCP listing tools return it as `structuredContent` beside `isError: true`.
- Added the codes `permission-denied`, `quota-exceeded` and `rate-limited` to a failure a server refused, read from the answer of the protocol that refused it (CalDAV, Google Calendar, Graph), and `item-unreadable` to an item a listing could not read, in the failure object, an `errors` entry and an `account check` row alike. calendula retries nothing, so `rate-limited` is the first 429 met.
- Added the code `item-invalid`, as cardamum's `card-invalid`: a CalDAV write answered 400 or 403 naming a precondition on the item's content (`valid-calendar-data`, `supported-calendar-data`, `valid-calendar-object-resource`, `supported-calendar-component`, RFC 4791 §5.3.2.1) or the iCalendar itself, under a hint saying calendula sends the item unchanged, the server's answer kept; a Google event write answered 400. A 403 whose plain body names the iCalendar read `permission-denied`, and the rest carried no code. Graph's refusals stay uncoded, naming no body recognisably.
- Added the codes `account-missing`, `account-default-missing` and `account-default-several` to the three failures of account resolution (an unknown `-a` name, no account marked default, several marked default), read from pimalaya-config's error, where they carried no code.
- Added `backend-not-built`: an account whose backend is one the build leaves out, its block loading unread, fails a unified command with that code, the message naming the cargo feature bringing it back (`--features gcal`) and, when the account carries a block of a built backend, the `--backend` reaching it; `account check` fails that backend's row with it. `account list` adds `notBuilt` beside `backends` (the text cell `gcal (not built)`), and `account capability` fails with `backend-not-built` as a unified command does. `--backend gcal` on a build without gcal is refused naming the feature, where it said `Invalid backend`.
- Added `proxy.url`, `proxy.username` and `proxy.password`, a SOCKS5 or HTTP proxy for every network backend of an account, and the same keys under `caldav`, `gcal` and `msgraph` to override it for one backend, as cardamum spells them. The password is a secret like any credential. Without either, `all_proxy` and `https_proxy` are read, honouring `no_proxy`, which CalDAV and Graph ignored.
- Added a `-V` or `--version` line to `repl`, printing the version the command line prints (pimalaya-cli 0.5.1).
- Added `pimdir queue list`, `pimdir queue show <ROW>` and `pimdir queue cancel <ROW>`, reading and retracting the store's queue as Himalaya's do: `queue list` prints, in the calendar `-k` names or in every calendar of the account, each creation and each reply or cancellation still queued, and every row the sync engine parked, each row in the family's one shape, `{row, kind, collection, linkId, itemId, producer, queuedAt, attempts, unconfirmed, error, summary}`, every key printed, `attempts` an unsigned integer; `queue show` where one row stands (`pending`, `parked`, `unconfirmed`, `applied`, `unknown`), an applied row with the event it acted on (`seq`) and the refusals standing there (`refusals`, `{collection, source, reason, refusedAt}`: a server that refused for good the change it led to, pimdir STORAGE §15.4); `queue cancel` drops one, asking unless `-y`, as the store's owner for that call alone, refused `store-owned` while a sync runs. `pimdir status` and `pimdir cancel` are unchanged.
- Added `description` and `color` to each collection `vdir list --json` prints, and a COLOR column to its text, as cardamum's `vdir list` has them.

### Changed

- Changed `gcal instances`, `gcal move` and `gcal quick-add` to `gcal events instances`, `gcal events move` and `gcal events quick-add`, after the Calendar API's Events resource, as the vendor namespaces name their resources; the flat names stay hidden commands doing the same. The JSON schemas `calendula-caldav-list` and `calendula-caldav-create` are now `calendula-caldav-propfind` and `calendula-caldav-mkcalendar`, their commands' new names. **Breaking** for a script reading those two schemas by name.
- Changed the refusal of a description or a colour on a Graph calendar, and of a calendar create, update or delete on pimdir, to fail with `capability-missing` before a credential is resolved or the store opened. Graph refused them after its token command ran, and pimdir after opening the store, failing first with the store's own error (`store-uncreated` on a store not created yet); the message now names the capability, the backend and the backend of the account serving it. **Breaking.**
- Changed `--if-match` on `event update` and `item update` over vdir, which was ignored, to be refused with `capability-missing`: a vdir item has no version to compare, and a guard dropped silently reads as one that held. **Breaking.**
- Changed `-c` to take one path per flag, repeated to merge several files (`-c public.toml -c private.toml`), never split on `:`; the `CALENDULA_CONFIG` variable is no longer read, `-c "$CALENDULA_CONFIG"` passing a path kept in a variable. **Breaking.**
- Changed the default configuration paths to load as layers: every existing one among $XDG_CONFIG_HOME/calendula/config.toml, $HOME/.config/calendula/config.toml and $HOME/.calendularc is deep-merged on top of the ones before, where the first one found was read alone, so an rc file left beside a config.toml now overrides it. **Breaking.**
- Changed account resolution to pimalaya-config 0.4: several accounts marked `default = true` are refused, naming them, where one of them was taken at random; an unknown `-a <NAME>` lists the accounts the configuration holds; no default names both ways of picking one. **Breaking.**
- Changed `-a default` to take the account named `default`, `default` being a name like any other, where it took the account marked `default = true`; without such an account it fails `account-missing`, saying to omit `-a` to take the account marked default. **Breaking.**
- Changed `auto` to stop on a backend the build leaves out, failing with `backend-not-built`, where it skipped the block and took the next backend the account configures, or failed saying no backend matched: one configuration reaches the same backend in every build. **Breaking.**
- Changed the configuration to refuse a key it does not know at the top level and in an account, naming it, where it ignored it, as every backend and option table already did: a misspelt `default` or `calendar` no longer loads silently. The block of a backend the build leaves out still loads, unread, so one file serves every build. **Breaking.**
- Changed the text output to plain text, with no border, no colour and nothing sized to the terminal. `event list` and `event find` print one line per event (id, start and end in the local zone, a whole-day event by its days with the end as its last one, summary, `-` for an empty cell) under a header naming the account with its backend, the calendar (and the UID for `event find`), where the page stands (`Page: 1, events 1 to 25`, `, the last`, `empty`, `all` when unpaged) and how to reach the next (`Next page: --page 2`); `item list` the same with its entity tag, size and id, and `gcal instances` with its id, status, start, end and summary. `calendar list`, `caldav list`, `gcal calendars`, `gcal free-busy` and `pimdir search` print records the way `git log` prints commits; `vdir list`, `gcal acl list` and `gcal colors` print aligned columns under an uppercase header, and `account list` one line per account with no header; `account check`, `caldav discover`, `gcal settings` and `pimdir status` print aligned `Label: value` fields. `event agenda` writes no terminal escape any more, where it wrote them even into a pipe, so it no longer highlights today or the days holding an event, the events under the grid naming their days. Every string a server or a file gave is defused of control and bidi characters. **Breaking.**
- Changed `-s 0` on `event list` and `item list` to list every item, where it listed none.
- Changed every write printing a confirmation to print a receipt instead of `{"message": ...}` under `--json`: `calendar create|update|delete` and `event|item delete` print `{id, queued}`, the item acted on and whether the write waits for a sync; `caldav create|delete`, `vdir create|rename|delete` and `gcal acl create|update|delete`, `gcal move` and `gcal quick-add` print `{id}`, a renamed collection's new id; `event|item build -o` prints `{path, size}`. The text form keeps its sentence, a queued delete now naming the item (`Event <id> deletion queued for the next sync`). Every command now has a schema under `json-schema`. **Breaking.**
- Changed piped `--json` output to print one value per line, each schema of `json-schema` to carry an `$id` naming the contract version, and added `calendula-error`, the schema of the JSON failure object. The abandoned arm of the `create` and `update` outputs, which `--json` never prints, is now `{}` rather than `null`, so each of their schemas is an object.
- Changed the clap help to plain, unwrapped text with the Pimalaya footer.
- Changed config.sample.toml into one account listing the options every account shares with their values, then one commented section per backend or provider (CalDAV, Google Calendar, Google CalDAV, Microsoft Graph, Fastmail, vdir, pimdir), each a working account once uncommented, its OAuth 2.0 token read from `ortie token show`, and every variant of it parsed by the test suite. The pimdir root now names neverest's shared store, ~/.local/state/neverest, and the copied sample no longer holds an active CalDAV account pointed at caldav.example.org.
- Changed `event update` to revise the event it writes: `DTSTAMP` is set to the time of the write in UTC, and `LAST-MODIFIED` to the same instant when the event carries one, whether the write came from field flags, a source or `-i`.
- Changed the composed-item check to also refuse a duration outside RFC 5545 3.3.6 (a `TRIGGER:P1H`, say) and a `LANGUAGE` or `ALTREP` on a property the RFC gives none.
- Changed the gcal and msgraph backends to mint an online meeting's join link as a `CONFERENCE` (RFC 7986) rather than their vendor property.
- Changed a Google account to accept an event giving its end as a `DURATION` (`event create --duration`, an invitation) instead of refusing it: the end is computed from the `DTSTART`, in its zone, and the event is stored with a `DTEND`. A negative `DURATION`, or one with hours after a date start, is still refused.
- Changed `event agenda` to name months and weekdays in the time locale, as cal does: `LC_ALL`, then `LC_TIME`, then `LANG`, with the English names of the POSIX locale when none is set or chrono does not know it. The event times under the grid are stamped in it too, and a month argument is read in English or in that language.
- Changed the local date windows (vdir, msgraph, pimdir) to keep a recurring event whose occurrences fall in them, not only one starting there.
- Changed CalDAV to list and address a calendar outside the home set by its path (Google's `/caldav/v2/<calendar>/events/`) rather than its last segment, to resend once a request the server closed the connection on, and to parse a multistatus 3.4 to 9 times faster, without quadratic copies and nested 64 levels at most.
- Changed an HTTP response whose head passes 64 KiB or whose body passes 256 MiB to fail the exchange with a typed error instead of growing the buffer without bound, and a connection closing mid-body to fail rather than spin. An iCalendar body nests 16 components at most.
- Changed a CalDAV `server` or `home` to take an ASCII host only, an internationalised one written in Punycode (`https://xn--bcher-kva.example/` for `bücher.example`): a non-ASCII host is refused when parsed, `home` when the configuration loads and `server` when the account connects, rather than converted. **Breaking.**
- Changed the pimdir backend to follow pimdir storage-draft-13, sync-draft-13 and search-draft-13 (io-pimdir 0.14). A store any earlier neverest wrote is refused as `store-version`, never migrated, the message saying what to run rather than to delete the store: sync it with that neverest until nothing is queued, upgrade neverest, move the store directory aside, then run `neverest init` and sync again. A store a newer engine wrote asks for a newer calendula, and a search index an earlier draft built is refused as `index-version` until the sync engine rebuilds it. **Breaking.**
- Changed the pimdir backend to read one account of the store: with `pimdir.account` unset, the account every collection is grouped under, and a store several accounts share is refused, naming them, rather than read with their calendars merged. **Breaking.**
- Changed the pimdir store reading to share one io-pimdir and one store schema with the rest of the set: a reply or cancellation the sync engine performed leaves a receipt, a store reconciled by the current neverest (no `probes` table) opens, opening a store creates the `items_by_sort_global` index a page across collections walks, a file item of the store (Annex A.7) reads as dateless, and an object whose blob entry is no body (a directory, a link, bytes not hashing to its name) lists with a warning, as a missing one does.
- Changed the pimdir listings to stream the store rather than load it: `event list`, `event find`, `item list` and `pimdir status` hold one batch of rows and the page they print instead of every body of the calendar, a page stops reading once full, and a body is not read where the stored summary already rules the item out, by its kind or, for `event find`, by its `UID`. The output is unchanged.
- Changed the pimdir write confirmations to say the write is queued for the next sync (`Event <id> queued for the next sync`, `... update queued ...`, `... deletion queued ...`) rather than created, updated or deleted, the JSON saying it in `queued`; a queued create's `id` is the link id it staged, which no command addresses before a sync applies it.
- Changed the pimdir `update_item` and `delete_item` to honour `--if-match`: a write naming a version the item no longer has is refused before anything is queued, with an error starting `Precondition failed:`. The queued update or delete names the body it was gated on, so the sync engine parks it, rather than apply it, when a sync replaced the body while it waited; `pimdir status` shows it.
- Changed the pimdir writes to refuse a to-do (`item` with a VTODO) into a calendar whose source states it takes none (`calendar.component.vtodo` `none` on the calendar or source-wide, pimdir STORAGE §15.6), before anything is queued, naming the source and pointing at `-k/--calendar`. A source stating nothing of it refuses no to-do.
- Changed the pimdir writes to be refused before they are queued when a source of the store does not support them, naming the capability, the source and why (pimdir STORAGE §15.6). A write a source supports only in part logs a warning.
- Changed a scheduled event refused for want of a source that notifies to name the way through: marking its `ORGANIZER` and `ATTENDEE` with `SCHEDULE-AGENT=NONE`.
- Changed `account check` on pimdir to probe a producer after opening the store, so a store a write cannot reach fails the check rather than the first write, with `store-busy` at once while a collector holds its staging lock; the check writes nothing into the store.
- Changed the dependencies to the current Pimalaya releases, every one from crates.io: ical-rs 0.7, io-gcal 0.5.1, io-http 0.8, io-msgraph 0.9.2, io-pimdir 0.14, io-vdir 0.3, io-webdav 0.11, pimalaya-cli 0.6, pimalaya-config 0.5 and pimalaya-stream 0.4. calendula links one ical-rs and one io-http where it linked two, and crossterm left the dependencies.
- Changed a CalDAV listing the server answers with anything but one whole multistatus (an HTML page, a cut document, an empty body) to fail, where it read as a shorter or an empty listing, and a delete answered by a multistatus naming failed members to fail (io-webdav 0.8).
- Changed a Google event holding more lines Google does not model than it stores beside an event (30,000 octets) to be written without the ones that do not fit, a warning naming them. A line wider than 1024 octets is stored whole, where it was left out without a word (io-gcal 0.4).
- Changed an Outlook recurring event whose `RRULE` Graph has no exact pattern for (a `BYMONTH` or a `BYMONTHDAY` the pattern has no field for, several numbered `BYDAY` elements, a weekday named twice) to be refused, where it was written as another series (io-msgraph 0.8).
- Changed the Outlook calendar listing to refuse a paging link leaving the Graph origin, as the event listings do.
- Changed the base64 and quoted-printable codecs linked through the dependencies (the CalDAV `Basic` header io-http encodes among them) to use the SIMD features of the CPU, detected at startup, rather than the portable code. The output is unchanged.
- Bumped rustls to 0.23.46, rustls-webpki to 0.103.16 and webpki-root-certs to 1.0.11, fixing RUSTSEC-2026-0346 to RUSTSEC-2026-0349, RUSTSEC-2026-0382 and RUSTSEC-2026-0383.
- Changed the exit codes to the family contract of pimalaya-cli: a command line clap refuses (an unknown flag, a missing argument, a bad value) exits 2 and, with `--json` before any `--`, prints on stdout the failure object coded `usage-invalid` (`{"code":"usage-invalid","error":"<clap message>","sources":[],"backtrace":null}`) where clap's rendering went to stderr; a result whose outcome is not a full success, a failed `account check`, exits 3, no longer 2. An MCP tool call its command refuses and a REPL line clap refuses carry `usage-invalid` too, and the `calendula-error` schema admits it. **Breaking.**
- Changed `calendar list`, `event list`, `event find`, `event agenda` and `item list` to exit 3 once they printed their result when `errors` is not empty. **Breaking.**
- Changed a vdir listing to list the other items where a file it could not read failed it whole, and a Graph listing to list the other events where a series master or its instances refused while read again failed it: each is an `errors` entry, a Graph event answered 404 there left out. A pimdir body the store cannot serve, which logged a warning or failed the listing, is an entry too, its item listed with no bytes, and so is a CalDAV row the server answered without the item's data (a 403 `permission-denied`) or a listing it truncated. **Breaking.**
- Changed `event list`, `event find` and `event agenda` to name an item whose iCalendar does not parse in `errors`, coded `item-unreadable`, where they skipped it without a word, `event list` counting it, and pimdir dropped a hydrated body that does not parse; `event list` counts the events it lists. **Breaking.**
- Changed the `event agenda --json` root to `{events, errors}`: every event under the grid as `{id, start, label}`, in the order the text prints them, `id` the item `event read` takes and `start` as `event list --json` prints it, where 0.2 printed a map of local start times to labels with no id. **Breaking.**
- Changed a missing item or calendar to fail coded, where the failure carried no code: `item-not-found` for `item read|update|delete` and `event read|update|delete` naming an id the calendar does not hold (a CalDAV or Google Calendar 404 or 410, a Graph 404 or malformed id, a vdir file or a pimdir item that does not exist), `calendar-not-found` for a vdir or pimdir calendar that does not exist.
- Changed `precondition-failed` to cover every backend: a CalDAV, Google Calendar or Graph 412 and Graph's `--if-match` mismatch failed with no code. A CalDAV create refused for `no-uid-conflict` carries `item-duplicate`.
- Changed `event create`, `event update`, `item create` and `item update` to print `queued` beside `id` under `--json`, where only the wording told a write queued for a sync from one made, and `calendar create` to print `{id, name, queued}`, `name` the display name given, as Himalaya's `mailbox create` does. **Breaking.**
- Changed `item list`, `item read` and `event read` to print an item as one row, `{id, calendarId, etag, contents, encoding}`: `contents` is the iCalendar text, `encoding` `utf-8`, or the base64 of bytes that are not valid UTF-8, `encoding` `base64`, both `null` in `item list` for an item carrying no bytes (a pimdir body not local yet), a read always carrying them. `item list` printed `contents` as an array of integers, `item read` printed `{contents, etag}` with no id and `event read` `{contents, events}`, both reads decoding a body that is not UTF-8 lossily; the text view still reads it, U+FFFD in place of what does not decode. **Breaking.**
- Changed an event's `uid`, `summary`, `description` and `location` to print `null` when it carries none, where they printed `""`. **Breaking.**
- Changed an event's `start` and `end` under `--json` to RFC 3339 with the offset the time was written at (`Z` for UTC), or `YYYY-MM-DD` for a whole day with an exclusive end, `null` when the time does not resolve, where they were iCalendar-spelled (`20261112T090000Z`): one spelling per time, the iCalendar one staying in the item's contents. **Breaking.**
- Changed `gcal free-busy --json` to print each calendar's errors under `queryErrors`, the API's `{domain, reason}` objects as it answers them, where `errors` held their reasons as strings: `errors` keeps one meaning across calendula, the item errors of a listing. **Breaking.**
- Changed `account list` to print pimalaya-cli's shared text, as Himalaya does: one line per account with no header row, `default` marking the default account, where it printed `NAME BACKENDS DEFAULT` and `yes`. The JSON is unchanged.
- Changed a `repl` or `mcp` line inside a session to be refused by pimalaya-cli 0.5.1, coded `usage-invalid`, where calendula's refusal carried no code.
- Changed an MCP call whose argument the command refuses to answer a sentence naming the argument as the tool does (``Invalid value `someday` for `from`: ...``, ``Missing argument `<name>` ``), where it answered clap's usage text.
- Changed `event build -o` and `item build -o` to print `Saved <size> bytes to <path>`, pimalaya-cli's `SavedOutput`, where they printed `Event successfully written to <path>`. The JSON is unchanged.
- Changed the refusal of a removed key or table (`todo`, `journal`, `table`, the top-level `account`, `calendar.list`, `event.list.table`, `item.list.table`, `caldav.discover`) to pimalaya-config's family sentence, ``The `<name>` table is removed, <why>: delete it, see MIGRATION.md``, where each had its own wording, most not naming MIGRATION.md.
- Changed `vdir.home-dir` to `vdir.root`, as `pimdir.root` names a store, its `~` and environment variables expanded as before; `vdir.home-dir` is refused by name (``The `vdir.home-dir` key is removed, …: rename it `vdir.root`, the path unchanged, see MIGRATION.md``). cardamum renames its key the same way. **Breaking.**
- Changed a command naming no calendar, on an account with no `calendar.default`, to fail coded `calendar-default-missing`, ``No default calendar found, name one with `-k <CALENDAR-ID>` or set `calendar.default` ``, where the failure carried no code.
- Changed the pimdir failures to carry every code Himalaya relays, by io-pimdir's spelling: `store-not-current`, `sync-version`, `store-owned`, `store-staging`, `store-in-flight`, `store-reset`, `conflict-unread`, `unsupported`, `no-performer` and `ambiguous-performer` join the vocabulary. A write a source does not support (an online meeting, a scheduled event, a to-do) fails `unsupported`, and a reply or a cancel no source can perform `no-performer` or `ambiguous-performer`, where they carried no code.
- Changed `event create` and `item create` to `event add` and `item add`, and `event find` to `event search`, the verbs of Himalaya; `create` and `find` stay hidden aliases. The MCP tools follow: `event_add`, `item_add` and `event_search`, where they were `event_create`, `item_create` and `event_find`, and so do the JSON schemas (`calendula-event-add`, `calendula-item-add`, `calendula-event-search`). **Breaking** for an MCP client.
- Changed `event search` to page its matches by `-p`, `-s` and `--page-token`, `nextPage` naming the next page while one follows, a series and its overrides counting once; with no `-s` every match is one page, as before.
- Changed `calendar create` to take the name alone, `calendar create <NAME> [--id <ID>]`, as cardamum's `addressbook create`: with no id, vdir names the calendar's directory after the name and CalDAV mints a UUID for its URL segment, a name possibly holding what a URL path cannot. The id before the name, `calendar create <ID> <NAME>`, is refused (exit 2): pass `--id <ID>`. Google and Graph, which mint the id, refuse `--id` with `capability-missing` before connecting, where they ignored it; `account capability` lists it as `calendar.create.id`. **Breaking.**
- Changed `calendar delete` and `calendar update` to take the calendar as an argument, `calendar delete <CALENDAR>`, as Himalaya's `mailbox delete`; `-k/--calendar` stays a hidden alias, and the MCP tools keep their `calendar` argument.
- Changed `--color` to take `-C`, as cardamum's, on `calendar create`, `calendar update`, `caldav create` and `vdir create`; `caldav create` and `vdir create` take `-n/--display-name` and `-d/--description` as `calendar update` does, `-D` a hidden alias of `-d`. **Breaking**: `-d` on those two now sets the description, not the display name.
- Changed `event delete` to take the event's id alone, as `event read` and `event update` do: a `UID` it no longer looks up fails `item-not-found`, naming `event search <UID>`, where it listed the calendar to find the event carrying it. **Breaking.**
- Changed `calendar list` to print as Himalaya's `mailbox list`: `Account:`, then one line per calendar, its id, `default` or `-`, its color, then its name, where it printed records with the description; and `item list` to print the id first, then the size and the entity tag. The JSON is unchanged.
- Changed a Google `event add` or `item add` naming a `UID` the calendar holds to fail `item-duplicate` before anything is written, naming the event that holds it, where `events.import` overwrote that event and exited 0; a CalDAV create answered 412 carries `item-duplicate`, the resource name being the `UID`'s, where it carried `precondition-failed`. Graph and vdir still take a second item of one `UID`. **Breaking.**
- Changed a CalDAV item read answered 400 to fail `item-not-found`, as a 404 does: iCloud answers 400 to a name without `.ics`, so `item read nope` and `event delete <uid>` failed with no code there.
- Changed `calendar update` on CalDAV to write only what it names, the colour first in a request of its own, so a server ignoring the colour fails the update before the name or the description changes, where the rest applied and the command exited 1; `calendar create --color` warns when the server kept no colour.
- Changed `event update --start` and `--end` naming no zone to be read in the zone of the event's `DTSTART`, where they made a zoned event floating, which Google and Graph refuse; `--time-zone` still wins.
- Changed `pimdir reply` to refuse an event naming no `ATTENDEE`, as Graph writes the account's own, where the reply was queued and the sync parked it; its refusals and those of `pimdir cancel` carry the new code `invitation-missing`.
- Changed `gcal acl update` to read the scope and its value from the rule id, `--scope` and `--value` now optional, where it sent no value (HTTP 400) unless `--value` repeated it; one disagreeing with the id is refused `usage-invalid` before the request, where Google answered 403 `permission-denied`.
- Changed `pimdir search` to page as every listing: `--json` prints `nextPage`, the next page number as a string or `null`, where it printed `more`, and `--page-token` takes it back, winning over `--page`; an event the store cannot read is an `errors` entry, the others listed and the command exiting 3, where the whole search failed; the text prints one line per hit (id, start, end, calendar, what matched, summary) under `Account:`, `Calendar:`, `Query:`, `Page:`, `Next page:`, `Incomplete:` and `Zones:`, where it printed records with their snippet, which `--json` keeps. **Breaking.**
- Changed a CalDAV calendar or item id to print percent-decoded where the decoding encodes back to the segment the server serves (io-webdav 0.10.0): a calendar created as `My Cal` lists as `My Cal`, no longer `My%20Cal`, an item `My%20Event.ics` as `My Event.ics`. A segment that would not be sent back as served (a `%2F`, an escape of a byte sent raw such as `%40`, lowercase hex) keeps its served form, and an id printed before still addresses its calendar or item, `calendar update` finding a calendar by either spelling. **Breaking.**
- Bumped pimalaya-config to 0.5.2: an unknown key's error lists no removed key.
- Changed `item build --json` to print `{contents, encoding}` and `event build --json` to add `encoding`, as cardamum's `card build` does, and both to hand over an item that is not UTF-8, in base64 under `--json` and as its bytes in text, where they failed.
- Changed the `-b/--backend` help to read as Himalaya's, listing `auto` and every backend with the one it pins, and an unknown `-b` to be refused listing the possible values. The `account`, `account list`, `gcal`, `pimdir` and `vdir` abouts read as Himalaya's: `Manage accounts defined in the TOML configuration file`, `List all accounts declared in the configuration`, `<API>-specific API`.
- Changed a missing item to fail with one sentence whatever the backend, as Himalaya words a missing message: ``Item `<id>` not found in calendar `<calendar>` `` on `item read`, `update` and `delete`, ``Event `<id>` not found in calendar `<calendar>` `` then how an event is named on `event read`, `update` and `delete`, where a read said ``Read item `<id>` from calendar `<calendar>` `` and a write the backend's own words. The code stays `item-not-found`, the backend's words in `sources`, its one source: a read no longer lists ``Read item `<id>` from calendar `<calendar>` `` there.
- Changed `calendar create` naming a calendar that already exists to fail coded `calendar-duplicate`, writing nothing, ``Cannot create calendar `<calendar>`: it exists already`` whatever the path (the id on vdir and CalDAV, the name on Graph), the server's answer in `sources`: on vdir, where it wrote its marker files over the existing calendar's and exited 0, and `vdir create` the same; on CalDAV, a MKCALENDAR answered naming `DAV:resource-must-be-null`, or 405 or 412 over a calendar the listing holds, where it failed uncoded; on Microsoft Graph, `ErrorFolderExists`. Google Calendar lets two calendars share a name. **Breaking.**
- Changed `gcal calendars`, `gcal acl list`, `gcal events instances` and `gcal settings` to print every item of every page as Google sends it under `--json`, under the same keys (`calendars`, `rules`, `instances`, `settings`), where they printed rows calendula shaped: a calendar's colour is `backgroundColor`, its reminders `{method, minutes}` objects, a rule's scope `{type, value}`, an instance's times `{dateTime, date, timeZone}` objects, and every key Google sends is kept (pimalaya-cli output.md, *Protocol commands are raw-faithful*). The text is unchanged. **Breaking.**
- Changed a vdir `item update` or `event update` given an iCalendar for an id the calendar does not hold to fail `item-not-found`, writing nothing, as on every other backend, where it created the item under that id. **Breaking.**

- Bumped pimalaya-cli to 0.6.0: `Next page:` prints after the listing, one blank line below the rows and above the errors, and the header values sit two spaces past the longest label.
- Bumped io-msgraph to 0.9.2, io-gcal to 0.5.1 and io-pimdir to 0.14.0 (pimdir draft-13).

### Removed

- Removed `-w/--max-width` from `calendar list`, `event list` and `item list`. **Breaking.**
- Removed the table and colour options: `table.preset`, `table.arrangement`, the `*-color` keys of `calendar.list.table`, `event.list.table`, `item.list.table` and `account.list.table`. A `table` or top-level `account` table left in a configuration is refused by name, and so are the `calendar.list`, `event.list.table` and `item.list.table` tables holding the colour keys; the `page-size` keys stay. **Breaking.**
- Removed the configuration wizard: the `configure` command (alias `wizard`), the offer a bare `calendula` and a command without a configuration made, the `wizard` cargo feature and the `calendula-configure` JSON Schema. Write the configuration from config.sample.toml; a command finding none fails naming the path it looked at and the sample, and a bare `calendula` prints the help. **Breaking.**
- Removed the `todo` and `journal` commands (`list`, `read`, `build`, `create`, `update`, `delete`) and their `todo.list` and `journal.list` options: calendula is the events CLI, to-dos moving to [unfog](https://github.com/pimalaya/unfog) and journal entries to [journal](https://github.com/pimalaya/journal). `item` still reads and writes any component raw. A `todo` or `journal` table left in a configuration, at the top level or under an account, is refused by name, pointing at the CLI that took it over. `item build -i` and `item create -i` given no source now name `calendula event`, `unfog` and `journal` as what composes an item. **Breaking.**
- Removed the `discover` route of the `caldav` block, which resolved a bare domain to a CalDAV root through RFC 6764 SRV records and the `.well-known` path on every run. Set `caldav.server` to the server's CalDAV root instead, such as Fastmail's `https://caldav.fastmail.com/dav/` or Google's `https://apidata.googleusercontent.com/caldav/v2/`; a `caldav.discover` left in a configuration is refused by name, the message naming `caldav.server` and MIGRATION.md. **Breaking.**
- Removed the io-pim-discovery dependency, which only `caldav.discover` used: the `caldav` feature links no DNS resolver any more.

### Fixed

- Fixed a Google or Outlook series losing its cancelled and moved occurrences on `event add`, `item add`, `event update` and `item update`: the master alone was written, so an EXDATE (on Outlook) and every override vanished with no word. The master is written, then each occurrence through the instance the server lists for it, deleted or updated; a create whose occurrence fails deletes the series again. What the server cannot hold is left out with a warning naming it (io-gcal 0.5.1, io-msgraph 0.9.2).
- Fixed a Google series written in another zone than its calendar's drifting an hour across a daylight-saving change: a weekly event at 09:00 in Paris read from a calendar in UTC listed at 10:00 after the change, where Google keeps 09:00. A boundary is read in the zone Google names (io-gcal 0.5.1).
- Fixed `calendar create --color` and `calendar update --color` writing a CalDAV calendar's colour as the inf-it `calendar-color`, which iCloud ignores: it is written as Apple's (`http://apple.com/ns/ical/`), which iCloud, Fastmail, Nextcloud and Radicale serve, and read from it, the inf-it one when it is absent (io-webdav 0.10.0).
- Fixed `calendar update -d ""` reporting success and keeping the description on CalDAV and Google: CalDAV now sends a `DAV:remove`, as for `-C ""`, which wrote nothing either, and Google an empty description.
- Fixed `event agenda` stamping an event written in UTC at its UTC time whatever `TZ` was: it stamps and files each event in the local zone, as `event list` shows it.
- Fixed a CalDAV to-do or journal entry named by a digest: a create names it after its `UID`, as an event.
- Fixed `event read` of a deleted Google event printing its cancelled stub: it fails `item-not-found`, and so does an update over it.
- Fixed an unset Outlook calendar colour printing `""`: it prints `null`, as on every backend.
- Fixed `pimdir search` and `pimdir count` reading every word after the query's first as a query word: `pimdir search x --page-size 3 --json` searched `x --page-size 3 --json`. An option is read wherever it stands; a query word starting with `-`, a negated term, goes after `--` (`pimdir search -- budget -lisbon`). **Breaking** for a `-` term written without `--`.
- Fixed `account check` passing a CalDAV account whose `caldav.home` pins the home-set whatever its credential: the check opened the connection and sent nothing. It now sends a PROPFIND Depth 0 on the home, so a credential the server refuses fails the row with the error the `caldav.server` route gives.
- Fixed a vdir calendar that does not exist read as an empty one: `event list -k nope` exited 0 listing nothing, `event create -k nope` created the calendar, and `calendar delete -k nope` failed with no code. Every command naming a calendar the vdir home does not hold, `vdir delete` and `vdir rename` included, now fails `calendar-not-found`, as on pimdir. An id that is not one directory name (empty, `.`, `..`, holding a separator, a trailing one included) is not found either, where `calendar delete -k ..` removed the directory holding the home, `calendar delete -k ""` the home itself and `calendar delete -k personal/.` every event of `personal`.
- Fixed `calendar create` and `vdir create` on vdir writing out of the home: `calendar create .. <name>` wrote a `displayname` file into the directory holding the home, `calendar create ../keep <name>` into the directory beside it, and `vdir create ../<id>` made a directory beside the home, each exiting 0. An id that is not one directory name (empty, `.`, `..`, holding a separator) is now refused before anything is written.
- Fixed a credential command prompting for a password (`systemd-ask-password`) hanging the run: it now reads the terminal when calendula runs in one (pimalaya-config 0.4.1).
- Fixed `event delete` reading a lookup the server refused (a 403, a 429, a dropped connection) as a missing id and falling back to a full listing: only `item-not-found` falls back, any other failure failing the command with its code.
- Fixed `event read` and `item read` printing the stored iCalendar's control characters as is, which could drive the terminal: the text view replaces them and the bidi controls, keeping tabs and line feeds and reading CRLF as LF, and `--json` keeps the stored text.
- Fixed a credential command run again each time a unified command reopened its backend (after a composer, say): the account's secrets now go through one resolver for the run, so a command is spawned once per account.
- Fixed a Google account refusing an event with neither `DTEND` nor `DURATION`: it is written as ending at its start, or the next day after a date start (RFC 5545 3.6.1), and no `DTEND` equal to a timed `DTSTART` is read back (io-gcal 0.3.1).
- Fixed an Outlook event's alarm that Graph's reminder cannot hold coming back with its lines scattered across the event after a write (io-msgraph 0.7.1).
- Fixed a CalDAV `server` written as a URL that fails to parse (an out-of-range port, a space in the host) being read as a bare authority, `https://` prefixed, and so sent to host `https`: it is refused.
- Fixed `event create --attendee` on a Google account mailing no invitation: an event naming attendees and no organizer, or the account as its organizer, is now inserted with `sendUpdates=all` rather than imported, keeping its `UID`. An event organized by another address is still imported, mailing nobody. ([#20](https://github.com/pimalaya/calendula/issues/20))
- Fixed a pimdir create or update losing its body: the body was written before the producer took the store's staging lock, so a collector running in between deleted it and the sync engine applied an event with no contents. The body is now staged through the producer that enqueues it, under its lock.
- Fixed a pimdir create whose `UID` the calendar already holds, or has queued, reported as created then parked by the sync engine: it is refused with `item-duplicate` before anything is queued.
- Fixed a pimdir create or update the store refused (a duplicate `UID`, a to-do into a calendar taking none, a scheduled event no source notifies for) leaving its body in the store's blob tree until a collection: the body is now written only with the queued action that pins it.
- Fixed a pimdir update made over the version it read (`event` or `item update` reading the stored item, or given `--if-match`) overwriting a change a sync landed while it waited in the queue: the queued action names that version, and the sync engine parks the update, shown by `pimdir status`, rather than applying it over the other change.
- Fixed `account check` exiting 0 when a backend failed: it now prints its report, in text and as the `--json` result, and exits 3, as cardamum and Himalaya do.
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
- Fixed a msgraph listing failing whole on a recurring series with no end or bounded over five years, Graph refusing an instances read past five years ([#31](https://github.com/pimalaya/calendula/issues/31)).

  A series is read within the range listed, or five years either side of today without one, in windows Graph accepts (io-msgraph 0.9.1), so an old series' recent exceptions are read too. A window Graph refuses makes that series an `errors` entry.
- Fixed a Google calendar or event id holding `#`, `?` or `/` naming another resource: the events of a subscribed calendar such as `en.usa#holiday@group.v.calendar.google.com` list and read (io-gcal 0.4).
- Fixed a Google listing failing whole on one value calendula does not know: `gcal calendars`, `gcal acl list` and `gcal instances` print an access role, a scope type or a status they do not know as the API spells it, and an update leaves such a status, transparency or visibility as the server has it (io-gcal 0.4).
- Fixed an update of a Google event whose reminders are off turning the calendar's default reminders back on (io-gcal 0.4).
- Fixed an Outlook account refusing an event with neither `DTEND` nor `DURATION`: it ends one day after a date start and at its start otherwise (RFC 5545 3.6.1) (io-msgraph 0.8).
- Fixed an Outlook series ending a day off away from UTC, the `UNTIL` of its `RRULE` having been sent as its UTC date (io-msgraph 0.8).
- Fixed an Outlook calendar or event id holding `/`, `?`, `#` or `%` naming another resource (io-msgraph 0.8).
- Fixed a folded line Google or Graph does not model coming back damaged after a write: with a stray CR at each fold on a Google account, cut at its first fold on an Outlook one, a `UID` over 71 octets included (io-gcal 0.4, io-msgraph 0.8).
- Fixed a CalDAV weak entity tag losing its closing quote, in a listing and in the `If-Match` a `--if-match` sends (io-webdav 0.8).
- Fixed a vdir listing failing on a directory or a dangling link named like an item, and on an item deleted while it lists. An item id or a new collection name holding a path separator is refused (io-vdir 0.3).
- Fixed an HTTP answer preceded by an interim `1xx` response read as that response, and the trailers of a chunked body starting the next answer of a reused connection (io-http 0.7).
- Fixed a CalDAV calendar or item id addressing another resource reaching the server: `calendar delete ..` could send a DELETE to the parent of the home set, `/` to the server root and `""` to the home set, `calendar update` a PROPPATCH there. An id that is empty or holds an empty, `.` or `..` segment now fails before the account's secret is resolved or anything connects, on every unified command naming one and on `caldav delete`: a calendar id `calendar-not-found`, an item id `item-not-found`, as vdir refuses an id that is not one directory name, and an id `calendar create` or `caldav create` gives a new calendar uncoded (io-webdav 0.9.1).
- Fixed a CalDAV item `event list`, `item list`, `event search` or `event agenda` reads losing its final CRLF, which iCloud and Fastmail escape, so it matches the body `item read` returns; a relative calendar or item id is sent percent-encoded, a `\` no longer read as a separator (io-webdav 0.9.1).
- Fixed `caldav report raw`, and `account check` on a pinned `caldav.home`, failing with `reached unexpected EOF` or `connection reset` when the server had closed the connection before answering: the REPORT and the check's PROPFIND go through io-webdav's client, sent again once on a fresh connection as every other CalDAV request is, and a refused raw REPORT carries the code any CalDAV refusal does, `permission-denied` on a 403 (io-webdav 0.11).

### Security

- Fixed the trace log showing sensitive response headers, and the PIN, passcode and password of a Google conference entry point (io-http 0.7, io-gcal 0.4).

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
