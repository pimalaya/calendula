---
cairn: delta
change: a-result-carries-its-errors
---

# Delta

Each requirement names its capability file in parentheses, as Himalaya's delta of the same name does.

## ADDED Requirements

### Requirement: A listing names the items it could not read
(commands) `calendar list`, `event list`, `event find`, `event agenda` and `item list` SHALL print an `errors` list beside their items, under `--json` and in text: one pimalaya-cli item error `{id, code, error, sources}` per calendar or item the command could not read, `[]` when there is none, in the order met. `id` SHALL be the item's id as the command prints ids, `null` when the backend named none, as for a listing the server truncated; `code` a code of the vocabulary or `null`; `error` the message; `sources` the backend's own reasons, outermost first. A non-empty `errors` SHALL make the command exit 3 once it printed its result. A listing that reads the whole calendar before cutting its page SHALL print its entries on the first page only; an item it listed but could not read (an iCalendar that does not parse, a pimdir body the store cannot serve) holds its place on a page, and its entry SHALL print on that page. `event list` SHALL count the events it lists, never an item it could not read.

#### Scenario: An item that does not parse
- GIVEN the vdir calendar `personal` holding the events `a` and `b` and one `.ics` file `c` that is no iCalendar
- WHEN `calendula --json event list` runs
- THEN it prints `a` and `b`, `"errors":[{"id":"c","code":"item-unreadable",…}]`, and the exit status is 3

### Requirement: A command's errors print at its end
(commands) In the text output, each `errors` entry SHALL print as one `Error:` line, `<id>  <code>  <error>` with `-` for a null id or code, last in the command's output: after the items (rows, records or the agenda's lines) and one blank line, the `Error:` lines aligned among themselves as pimalaya-cli's `Columns` aligns cells, each value sanitized. Nothing SHALL print, the blank line included, when `errors` is empty.

### Requirement: A server's refusal carries its code
(backends) A failure a server refused SHALL carry a code of the vocabulary Himalaya shares, read from the typed error of the protocol that answered it, the failure object, an `errors` entry and an `account check` row alike:

| Code | When |
|---|---|
| `permission-denied` | an HTTP 403 that is no rate limit (Google Calendar, Graph); a CalDAV 403 whose body names no precondition, or `need-privileges` (RFC 3744 §7.1.1) |
| `quota-exceeded` | an HTTP 507; a CalDAV body naming `quota-not-exceeded` (RFC 4331 §6); Graph's `ErrorQuotaExceeded` |
| `rate-limited` | an HTTP 429, or a Google Calendar 403 naming a rate limit or a quota; calendula retries nothing, so the first one met |
| `item-not-found` | an item request answered 404 or 410 (CalDAV, Google Calendar), 404 or 400 `ErrorInvalidIdMalformed` (Graph); a vdir item file or a pimdir item that does not exist |
| `calendar-not-found` | a calendar named that does not exist (vdir, pimdir) |
| `precondition-failed` | an HTTP 412 (CalDAV, Google Calendar, Graph), Graph's `--if-match` mismatch read before the write, a pimdir version that moved |
| `item-duplicate` | a CalDAV create refused for `no-uid-conflict` (RFC 4791 §5.3.2), a pimdir create naming a `UID` its calendar holds or has queued |

A refusal none of these name SHALL carry no code, its reason kept in `sources`. A CalDAV 403 naming another precondition (`valid-calendar-data`, `supported-calendar-component`) is an item the server would not take, no matter of rights, and SHALL carry none.

#### Scenario: A read-only Graph calendar
- GIVEN a Graph calendar the account may read but not write
- WHEN `calendula --json event delete <id>` runs
- THEN it exits 1 printing a failure object coded `permission-denied`

### Requirement: Each backend names the items it could not read
(backends) A backend SHALL report an item it could not read as an `errors` entry and SHALL never drop it silently, with a warning alone, nor fail the listing for it:

- every backend: an item whose iCalendar does not parse SHALL be an `item-unreadable` entry in `event list`, `event find` and the agenda, no event row printed for it; `item list` SHALL list it as it lists any item, with no entry. A kind or a range filter applied after parsing SHALL keep such an item, whose kind and dates cannot be read;
- vdir: an item file that cannot be read SHALL be an `item-unreadable` entry naming the item, its error in `sources`; an item removed while it lists SHALL be left out with no entry;
- Graph: an event whose series master or instances the server refused while they were read again SHALL be an entry coded by the refusal (`item-unreadable` when the refusal carries no code), the other events listed; one answered 404 SHALL be left out with no entry;
- pimdir: a body that cannot be read, or that the blob tree does not hold, SHALL be an `item-unreadable` entry, the item listed with no bytes; a hydrated body that does not parse SHALL be an entry where the kind filter dropped it; an item whose body is not local yet SHALL be no entry;
- CalDAV: a row the server answered without the item's data (refused whole, its `calendar-data` refused, or missing) SHALL be an entry naming the item, coded from its status, a 403 `permission-denied`, else `item-unreadable`, each refused property in `sources`; a listing the server truncated (a 507 row) SHALL be an entry with a `null` id and no code saying the listing is incomplete;
- CalDAV, Google Calendar: an answer or a page that fails SHALL fail the listing whole.

## MODIFIED Requirements

### Requirement: A JSON error carries a stable code
(commands) A failure a caller is expected to act on SHALL carry a stable code, printed under `--json` as a `code` string beside `error`, `sources` and `backtrace`, and in an `errors` entry beside `id`, `error` and `sources`. The code SHALL be found anywhere in the error chain, so added context does not hide it, or read from the typed error of the protocol that refused ("A server's refusal carries its code"). A failure with no code SHALL print `"code": null`. The plain output SHALL stay unchanged. Codes are kebab-case and never renamed; `error` stays free wording. Every code SHALL be declared in every build, whatever the backends compiled in.

| Code | Raised when |
|---|---|
| `capability-missing` | the backend the account resolves to does not serve a capability the command reaches; nothing was read, opened or connected |
| `stdin-taken` | a command run inside a session (`repl`, `mcp`) would read its iCalendar from stdin, which carries the session's commands; nothing was read |
| `item-not-found` | the item a command names is not in the calendar: the failure of `item read`, `item update`, `item delete`, `event read`, `event update` or `event delete` |
| `calendar-not-found` | the calendar a command names does not exist; nothing was read or written |
| `item-unreadable` | the backend answered for an item calendula could not read: its file could not be read (vdir), its iCalendar did not parse, the server refused it while it was read again (Graph), its body could not be read or is missing from the store (pimdir); an `errors` entry |
| `permission-denied` | the server refused the request to the account (see "A server's refusal carries its code") |
| `quota-exceeded` | the write would exceed the account's quota |
| `rate-limited` | the server throttled the request; calendula retries nothing |
| `body-pending` | a listed item's body is not local yet (pimdir); a sync brings it |
| `precondition-failed` | a write's `--if-match` names a version the item no longer has, on any backend that takes one |
| `occurrence-unsupported` | `pimdir reply` or `pimdir cancel` names an occurrence its performer cannot act on alone |
| `item-duplicate` | a create names a `UID` its calendar holds: a CalDAV `no-uid-conflict`, a pimdir calendar holding or queuing it |
| `store-uncreated` | the pimdir store has no schema yet: its sync engine creates it |
| `store-version` | the pimdir store is of another format version than calendula's: an earlier sync engine's is reinstalled, a newer one's needs calendula upgraded |
| `store-foreign` | the pimdir database is not a pimdir store: another program wrote it |
| `store-corrupt` | the pimdir store's two version stamps disagree, or its schema is not the canonical one |
| `store-hash-algo` | the pimdir store names its bodies by a hash this calendula does not know: upgrade calendula |
| `store-busy` | another process holds the pimdir store; retry |
| `store-exposed` | a path of the pimdir store is a link or another user can reach it: restrict it |
| `lock-replaced` | the lock file of the pimdir store was removed or replaced under a write: nothing was queued; retry |
| `flag-invalid` | a pimdir write carries a flag the store cannot hold |
| `action-malformed` | a pimdir queue action's payload is one its schema refuses |
| `index-missing` | the pimdir store has no search index yet: its sync engine builds it |
| `index-outdated` | the pimdir search index was built otherwise or from another store: its sync engine rebuilds it |
| `index-version` | the pimdir search index is of another draft or shape: its sync engine rebuilds an earlier draft's, and leaves any other |
| `index-zone-unknown` | the pimdir search index reads floating times, or the search reads days, in a zone the time zone database does not know |
| `index-fts5-missing` | the SQLite calendula links lacks FTS5 or is older than 3.43 |
| `query-refused` | the pimdir search query is malformed or names an unknown field or value |
| `query-unanswerable` | the pimdir search index could not answer an accepted query |
| `usage-invalid` | the command line does not match the grammar (an unknown flag or command, a missing argument, a bad value), raised by pimalaya-cli; nothing ran |

#### Scenario: A stale version
- GIVEN a pimdir event whose body changed since it was read
- WHEN `calendula --json event delete --if-match <old etag>` names it
- THEN the command exits 1, queues nothing and prints `{"code":"precondition-failed","error":"Precondition failed: …","sources":[],"backtrace":null}`

#### Scenario: An unknown item
- GIVEN the vdir calendar `personal` holding no item `nope`
- WHEN `calendula --json item read nope` runs
- THEN the command exits 1 and prints a failure object coded `item-not-found`

### Requirement: The JSON contract is described and versioned
(commands) `json-schema` SHALL describe the failure object under the key `calendula-error`, its `code` typed by the code vocabulary or `usage-invalid`, the code pimalaya-cli raises itself, and the item error of an `errors` list under `calendula-item-error`, its `code` typed by the code vocabulary. Every generated schema SHALL carry an `$id` naming the contract version: the major version of calendula, or `0.<minor>` before 1.0. Every registered payload SHALL be a JSON object, checked by a test. Piped `--json` output SHALL print one value per line.

### Requirement: A refused command line exits 2
(commands) The command line SHALL be parsed by pimalaya-cli's `clap::parse`: help and version print as clap prints them, and a command line clap refuses (an unknown flag or command, a missing argument, a bad value) SHALL exit 2 before any configuration is read. With a `--json` word before any `--`, it SHALL print on stdout the failure object coded `usage-invalid`, its `error` clap's message without its `error: ` prefix nor colour, and nothing on stderr; otherwise clap's rendering on stderr and nothing on stdout. Exit 1 stays a failure object and 3 a printed result that is not a full success: `account check` with a failed backend (see "account check fails when a backend fails") and a listing whose `errors` is not empty.

#### Scenario: A misspelt flag
- WHEN `calendula --json --nope calendar list` runs
- THEN it exits 2 and prints `{"code":"usage-invalid","error":"unexpected argument '--nope' found …","sources":[],"backtrace":null}` on one line
- AND `calendula --nope calendar list` exits 2, printing clap's `error: unexpected argument '--nope' found` on stderr and nothing on stdout

### Requirement: Text output is plain text
(commands) Its first paragraph and the records list that ends it stay as written; its second and third paragraphs read:

`event list` and `event find` SHALL print as Himalaya's `envelope list` does. Above the events, `Account:` (the account name, then the backend it resolved to in parentheses), `Calendar:` and, for `event find`, `UID:`, then `Page:` (the page number, `all` for an unpaged listing, then the events the page holds, `event N` or `events N to M` followed by `, the last` when no page follows, or `empty`), and while a page follows `Next page:` and the `--page` reaching it, these values starting in one column, two spaces after `Next page:`; then a blank line. Then one line per event through pimalaya-cli's `Columns` with no header row: its item id, its start and its end (Columns follow the kind), then its summary, last, `-` for an empty cell. Under them SHALL print only the command's errors ("A command's errors print at its end"). The events `Page:` counts are the calendar's events, the resources a page holds: a series counts once, however many lines its overrides or its occurrences in a window print. `, the last` and `Next page:` SHALL be exact on every backend: the unified client asks for one item past the page. `-s 0` lists every item, unpaged.

`item list` SHALL print the same header above one line per item: its entity tag, its size, then its id. `gcal instances` SHALL print it above one line per instance: its id, its status, its start and end as the server spells them, then its summary, `Event:` naming the series in the header, unpaged.

#### Scenario: An event list
- GIVEN the vdir account `perso` whose calendar `personal` holds the timed events `a`, `b` and `c`, read in UTC
- WHEN `calendula event list -s 2` prints them
- THEN it prints `Account:    perso (vdir)`, `Calendar:   personal`, `Page:       1, events 1 to 2` and `Next page:  --page 2`, then a blank line, then `a  2026-11-12 09:00  2026-11-12 09:30  Standup` and the line of `b`, aligned, with no border and no colour
- AND `calendula event list -s 2 -p 2` prints `Page:       2, event 3, the last` above the line of `c`

### Requirement: An agenda shows every event at an instant
(commands) The agenda SHALL render every event whose start falls at a given instant, never one of them, and SHALL order two events at one instant totally and stably, by label then by item id, so the same calendar renders the same way twice.

The events at one instant SHALL print under one time column rather than repeating it, so a duplicated pair reads as two events rather than as two rows that happen to share a time.

Two events may legitimately start at the same moment: two unrelated meetings at 09:00, and, since a collection may hold two resources under one `UID` (pimdir STORAGE §9), two copies of what a server considers one identity.

A view keyed by the instant alone shows the last one written and reports nothing, which loses exactly the event the store went to the trouble of keeping.

The `--json` payload SHALL carry the same multiplicity, so a consumer reading it is not told there was one event where there were two: under `starts`, it maps each start datetime to the list of the labels starting at it, in the same order the text renders them, and under `errors` it lists the items the agenda could not read ("A listing names the items it could not read").

No second shape SHALL be kept beside it, since one that still dropped an event would leave the defect reachable.

### Requirement: An event is deleted by its id or its UID
(commands) `event delete <EVENT-ID>` SHALL take the id `event list` reports or the event's iCalendar `UID`. It SHALL try the id first, by a read, then, only when that read answers `item-not-found`, list the calendar's events and look for an item of that id, then for the items carrying that `UID`; any other failure of the read SHALL fail the command with its code. A read answering `body-pending` (pimdir) names the item all the same. One item SHALL be deleted whole; several carrying the `UID` SHALL be refused, naming their ids; none SHALL be a clear miss, coded `item-not-found`.

### Requirement: pimdir is an availability-aware cache
(backends) An item whose body is not local (`level < Full`, no stored object) SHALL still list, carrying no bytes. `get_item` on such an item SHALL report a clear "body not fetched" state, the cue to sync, not a data-loss error.

A range filter SHALL still apply to it, read off the stored `text/calendar` summary rather than off bytes that are not local: a cache that hid its own undownloaded items from a date window would answer a different question than the one asked.

An item naming an object the blob tree holds no body for is a different state: the store is inconsistent rather than partially synced. The blob file may be absent, or its entry no body at all (a directory, a link, bytes that do not hash to its name), which the store reads as absent (pimdir STORAGE §5, §7). It SHALL still list, carrying no bytes, and the listing SHALL name it in its `errors`, coded `item-unreadable`, since the row renders the same as an unhydrated one and `get_item` refuses it outright.

### Requirement: A tool's output is the command's payload
(mcp) A successful call SHALL return the command's JSON value as `structuredContent` and as text, described as `outputSchema` by its registered schema when that schema's root is typed `object`. `event_create`, `event_update`, `item_create` and `item_update`, whose schema is one of two shapes (the write, or an abandoned composer edit), SHALL carry no `outputSchema`. A call ending with a partial outcome (a listing whose `errors` is not empty, an `account check` with a failed backend) SHALL return its result as `structuredContent` beside `isError: true`. A refused or failed call SHALL return `isError: true` with the `calendula-error` object and no `structuredContent`.

### Requirement: CalDAV backend
(backends) CalDAV SHALL adapt io-webdav's RFC 4791 surface over a connected client whose calendar home-set is already resolved.

Item ids SHALL be the resource names the server returned, verbatim: io-webdav neither appends nor strips a file extension, so an id a listing showed addresses the same resource on every verb.

A create SHALL propose a resource name derived from the item's own UID when that UID is URL-safe, falling back to a content digest, and SHALL keep the id the server reports in its `Location` header when it names the resource itself, else the last segment of the URL io-webdav sent, percent-encoded, which is the form a listing returns.

A listing the server answers with anything but one whole multistatus (an HTML page, a cut document, an empty body) SHALL fail, never read as a shorter listing, and a delete answered by a multistatus naming failed members SHALL fail.

A listing SHALL name, in its `errors`, each row the server answered without the item's data and a listing the server truncated ("Each backend names the items it could not read"), on the first page only: the whole calendar is read before the page is cut, and such a row holds no place on a page.

## REMOVED Requirements

None.
