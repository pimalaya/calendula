---
cairn: change
id: a-result-carries-its-errors
status: landed
created: 2026-11-14
---

# A result carries its errors

## Why

Himalaya landed "a result carries its errors" on 2026-11-14 (himalaya/cairn/changes/a-result-carries-its-errors): a command listing or acting on several items prints an `errors` list beside its result, one pimalaya-cli item error `{id, code, error, sources}` per item it could not serve, and exits 3 when the list is not empty. The family vocabulary gained `permission-denied`, `quota-exceeded` and `rate-limited`, beside Himalaya's `message-not-found`, `message-unreadable` and `mailbox-unreadable`. Neverest followed the same day (neverest/cairn/changes/a-refusal-carries-its-code): a server's refusal is coded by a reader per protocol walking the error chain. pimalaya-cli's log of the same name says calendula and cardamum adopt `errors` "when they print several items that can fail one by one".

Calendula's listings are `calendar list`, `event list`, `event find`, `event agenda` and `item list`, and an item they cannot serve is lost or takes the listing with it:

- **Dropped silently.** An item whose iCalendar does not parse yields no event (src/shared/event/expand.rs:74-77, :107-110): `event list` (src/shared/event/list.rs:124-133), `event find` (find.rs:62-66) and the agenda (agenda.rs:299-303) skip it with no warning, `event list` still counting it (list.rs:139). A pimdir body that does not parse is dropped by `is_kind` (src/pimdir/backend.rs:1094-1097). io-webdav drops a CalDAV row the server refused per href (403, 404) before calendula sees it.
- **A warning alone.** A pimdir item naming a body the blob tree does not hold lists empty with a `warn!` (src/pimdir/backend.rs:611-625), which the spec ("pimdir is an availability-aware cache") asks for.
- **The whole listing fails.** A vdir file that cannot be read (src/vdir/backend.rs:115-131, io-vdir's `list_items`), a Graph series master or its instances refused while re-read (src/msgraph/backend.rs:184-189), a pimdir blob read error (src/pimdir/backend.rs:613).
- **No code.** No failure says whether the server refused the account (403), its quota (507) or the rate (429). An item or calendar not found is an uncoded `bail!` (src/pimdir/backend.rs:603 and :530, src/shared/event/delete.rs:123, src/vdir/backend.rs:91, src/caldav/backend.rs:85). `precondition-failed` is pimdir's alone: a Graph `--if-match` mismatch is an uncoded `bail!` (src/msgraph/backend.rs:250-253, :276-279), and a CalDAV or Google Calendar 412 an uncoded chain. CalDAV's `no-uid-conflict` refusal (io-webdav's `DuplicateUid`) carries none of `item-duplicate`.
- **An error mistaken for absence.** `event delete` resolves its id with `client.get_item(...).is_ok()` (src/shared/event/delete.rs:91), so a 403, a 429 or a dropped connection reads as "not this id" and falls back to a full listing.

The libraries move first: the family moved to pimalaya-stream 0.4 on 2026-11-13, and pimalaya-cli 0be1b1f carries the `ItemError` this change prints.

## What

Four phases, each one commit, in this order. Phases 1 to 3 need nothing outside this repository but Phase 0; Phase 4 waits on io-webdav.

### Phase 0, outside this repository (blocking Phase 1)

io-pim-discovery's change stream-0-4 (io-pim-discovery/cairn/changes/stream-0-4) lands and is pushed: calendula's CalDAV discovery hands it a `Tls` (src/caldav/client.rs:127), so the `caldav` feature cannot take pimalaya-stream 0.4 before it. Its commit hash is the rev Phase 1 pins. If the owner has released io-webdav 0.9, io-gcal 0.5, io-pim-discovery 0.13 or pimalaya-cli 0.4.1 by then, Phase 1 takes the release instead of the pin.

### Phase 1, the libraries

- Cargo.toml: `pimalaya-stream` 0.4, `io-http` 0.8, `io-msgraph` 0.9; dev `io-oauth` 0.7; `[patch.crates-io]` (sorted, no comment, CONTRIBUTING.md's rule; this change says what each waits on): `io-gcal` dbb4481cd77ef65ca451420cb4e087fdccfe6fdb and `io-webdav` 185ab63be5859c5ef6115bef97e4df5cb1187973 (their stream 0.4 bumps, until io-gcal 0.5 and io-webdav 0.9), `io-pim-discovery` at the Phase 0 commit (until 0.13), `pimalaya-cli` 0be1b1f08d19ccb9d16ee8e1d4266c27c96e0cdd (eb37962 plus `ItemError` and the registry spare, additive only, until 0.4.1). Then `cargo update`, which takes pimalaya-config 0.4.1 (a prompting secret command reads the terminal again).
- src/config.rs:559 (`TlsConfig::into_tls`, :546-561): `server: self.cert.map(|cert| TlsServer { cert })`, `client: None`. The `tls.cert` key keeps its name and meaning (config.sample.toml:28, :39, :57).
- Expected compile breaks: that field only. io-http 0.8, io-oauth 0.7 and io-msgraph 0.9 have byte-identical sources to their predecessors; io-webdav 185ab63 and io-gcal dbb4481 change only their dependencies; calendula already implements pimalaya-config 0.4's `TomlConfig`. io-http's `HttpAuthBasic` and `HttpAuthBearer` (src/caldav/client.rs:13, :148-160) must be the io-http io-webdav uses, which the one-copy check proves.
- `cargo tree -d --all-features -e normal` lists no family crate twice.
- CHANGELOG: the dependency entry under Changed (about :67) follows; a Fixed line for the prompting secret command.

### Phase 2, a refusal and a missing item carry their code

- src/error.rs gains, declared in every build:
  - `permission-denied`, `quota-exceeded`, `rate-limited`, spelled as Himalaya's;
  - `item-not-found`: the item id a command names is not in the calendar (`item read|update|delete`, `event read|update|delete`), or an `errors` entry;
  - `calendar-not-found`: the calendar named does not exist;
  - `item-unreadable`: the backend answered for an item calendula could not read; an `errors` entry, or the failure of `item read`/`event read`;
  - `calendar-unreadable`: a calendar listed whose properties could not be read; kept only if Phase 3 meets it (a vdir collection whose metadata file cannot be read is the candidate, unverified), otherwise left out.
- `precondition-failed` widens from pimdir to every backend: a CalDAV or Google Calendar 412, and Graph's `--if-match` mismatch (src/msgraph/backend.rs:250-253, :276-279). `item-duplicate` widens to CalDAV's `no-uid-conflict` (io-webdav's `WebdavSendError::DuplicateUid`).
- One reader per protocol beside its client, `pub(crate) fn refusal_code(cause: &(dyn std::error::Error + 'static)) -> Option<ErrorCode>`, and in src/error.rs the chain walk `error::code(err)` = `code_of(err)` else the first cause a reader codes (neverest/src/error.rs). Readers mirror neverest's:
  - CalDAV (`WebdavClientStdError`, its `Send`, `WebdavSyncCollection` and `WebdavFollowRedirects` statuses): a 507 or a body naming `quota-not-exceeded` (RFC 4331 §6) `quota-exceeded`; 429 `rate-limited`; 403 `permission-denied` when its body names `need-privileges` (RFC 3744 §7.1.1) or no precondition; `DuplicateUid` `item-duplicate`; 412 `precondition-failed`; 404 or 410 on an item request `item-not-found`.
  - Google Calendar (`GcalClientStdError::Send(GcalSendError::Api(..))`): neverest's `google_code` copied with the parts of `google_refusal` it reads (429, or a 403 naming a rate limit or a quota, `rate-limited`; another 403 `permission-denied`); 412 `precondition-failed`; 404 or 410 on an event request `item-not-found`.
  - Graph (`MsgraphClientStdError::Send(send)`, `send.api()`): `ErrorQuotaExceeded` or 507 `quota-exceeded`, 429 `rate-limited`, 403 `permission-denied`, 412 `precondition-failed`; 404, or 400 `ErrorInvalidIdMalformed`, `item-not-found` on an event request.
  - vdir: a missing item file `item-not-found` (src/vdir/backend.rs:91; io-vdir 0.3's variant read from `VdirClientError`), a missing collection `calendar-not-found`.
  - pimdir: src/pimdir/backend.rs:603 `item-not-found`, :530 `calendar-not-found`.
  - src/shared/event/delete.rs:123 `item-not-found`; src/caldav/backend.rs:85 the code its cause names.
- `event delete` (src/shared/event/delete.rs:91) falls back to a full listing on `item-not-found` alone; any other failure of the lookup fails the command with its code.
- `account check` rows read their `code` through `error::code` (src/account/check.rs:254).
- No retry is added: calendula retries nothing (cairn/log/2026-11-13-io-family-releases.md, "backs off nowhere"), so `rate-limited` codes the first 429 met. The code table says so.

### Phase 3, a listing names what it could not read

- `calendar list`, `event list`, `event find`, `event agenda` and `item list` print an `errors` list beside their items, under `--json` and in text: one `pimalaya_cli::error::ItemError<ErrorCode>` (alias `ItemError` in src/error.rs) per calendar or item the command could not read, `[]` when there is none, in the order met. `id` is the item's id as the command prints ids, `null` when the backend named none. A non-empty `errors` makes the command return `PartialOutcome::default()` once printed: exit 3.
- What becomes an entry:
  - every backend: an item whose iCalendar does not parse, met by `event list`, `event find` or the agenda, is an `item-unreadable` entry naming its id and the parse error, and no event row; `event list` counts the events it lists, not the items it read. `item list`, which projects no event, lists such an item as today with no entry;
  - vdir: `list_items` is replaced by io-vdir 0.3's `list_readable_items`, each file it could not read an `item-unreadable` entry with the item's id (as calendula names vdir items) and the `io::Error` in `sources`; a vanished file stays left out with no entry;
  - Graph: a series master or its instances refused while re-read (src/msgraph/backend.rs:184-189) is an entry coded by its refusal, the other events listed; a 404 there leaves the event out with no entry (it went away since the listing); `unique()` (:370-390) keeps failing whole;
  - pimdir: a blob read error (:613), and an item naming a body the blob tree does not hold (:611-625, today a warning), are `item-unreadable` entries, the item listed empty as today; an item not hydrated yet stays a normal state, no entry; a hydrated body that does not parse (:1094-1097) is an `item-unreadable` entry instead of being dropped;
  - CalDAV and Google Calendar: an answer or a page that fails fails the listing whole (Google Calendar deserializes each page whole, src/gcal/backend.rs:162-209).
- The agenda's `--json` root becomes an object, `{"starts": {<instant>: [<label>…]}, "errors": [...]}`: today it is a bare map of start instants (src/shared/event/agenda.rs:1088, :1107), which an `errors` key would collide with. The map keeps its keys, values and order under `starts`. The text agenda prints its `Error:` lines last, after its `stamp: label` lines and one blank line.
- Paging: a backend reading the whole calendar before cutting its page (CalDAV, Graph, vdir; `event find` on every backend) prints its entries on the first page only, as Himalaya's Maildir listing does; pimdir prints an entry on the page that held its item.
- Text: a list prints its paging in its header, and a command's errors print at the end of its output (owner decisions 2026-11-14: no list footer; the errors are the command's own footer, not the list's). `Listing::write` (src/shared/listing.rs:49-110) moves its paging fields (`Page:`, `Next page:`) from below the rows into the header, for `event list`, `event find`, `item list` and `gcal instances`, which shares it. Each entry prints as an `Error:` line at the very end, after the rows and one blank line, `<id>  <code>  <error>` with `-` for a null id or code, the `Error:` lines aligned among themselves; `calendar list` (`Records`, src/shared/calendar/list.rs:52-78) and the agenda the same after their lines. Nothing prints, the blank line included, when `errors` is empty. A later change moves these lists onto pimalaya-cli's shared listing module.
- Schema: `json-schema` registers the item error under `calendula-item-error` (src/json_schema.rs:206); `every_command_is_described` keeps passing through pimalaya-cli 0be1b1f's spare. `errors` is a required key of each listing output.
- MCP: `calendar_list`, `event_list`, `event_find` and `item_list` return their result with its `errors` as `structuredContent` beside `isError: true` on exit 3, which `ToolResult::printed` already does (src/mcp.rs:328); the server instructions (src/mcp.rs:62-64) say a listing may carry `errors`.
- REPL: `Session::run_unified` (src/session.rs:44-66) keeps its client after a partial outcome, dropping it on any other `Err` as today.

### Phase 4, after io-webdav (not part of the parallel run)

io-webdav's change a-listing-returns-what-it-could-not-read makes `CaldavItemList` return the rows it could not read beside the items (a breaking change of its output). Once pushed and pinned, a CalDAV row refused per href is an `item-unreadable` entry coded from its row status (403 `permission-denied`), and a listing the server truncated is an entry with a `null` id and no code.

### Docs

CHANGELOG Unreleased: Added the codes and `errors`; **BREAKING** (as calendula writes it): listings exit 3 with an `errors` list when an item could not be read, a vdir or Graph listing lists the other items where it failed whole, an unparseable item is named rather than skipped, and the agenda's JSON root moves under `starts`; Changed: not-found failures carry `item-not-found` and `calendar-not-found`, `precondition-failed` covers every backend, a list prints its paging above its rows; Fixed: `event delete` no longer reads a refused lookup as a missing id. MIGRATION.md, "0.2.x -> unreleased", "JSON output" (:66-73): `errors`, exit 3, the agenda's `starts`. README: an exit status line if it has none. The src/main.rs header (:175-181) names exit 3 for a listing with errors.

## Decisions

- **Same id as Himalaya's and pimalaya-cli's change**, so the family's history reads across repositories.
- **Listings only.** No unified calendula command takes several ids, so a write keeps failing whole, coded. The protocol namespaces (`caldav`, `gcal`, `pimdir`, `vdir`) keep printing what their protocol answered: `gcal free-busy`'s per-calendar `errors` strings (src/gcal/free_busy.rs:83-87, :113) stay Google's answer, nested in a calendar, not the shared noun.
- **Item codes named by calendula's nouns**: `item-*` and `calendar-*`, as Himalaya names `message-*` and `mailbox-*`, and as calendula already names `item-duplicate`.
- **An unparseable item is named, not listed as an event**: an event row needs a projection; `item list`, which needs none, still lists it.
- **A pimdir body the store cannot serve becomes an entry, an unhydrated one does not**: the first is an inconsistent store, the second the normal state of a young one.
- **The agenda's JSON gains a root object.** Its single-shape rule ("No second shape SHALL be kept") stays: the map moves under `starts`, no second agenda shape is kept.
- **Paging in the header, errors at the end**: the owner decided on 2026-11-14 that a list prints no footer, so the paging moves above the rows; a command's errors are no part of the list but the end of the command's output, so they print last.
- **No retries**, so `rate-limited` is the first 429 met; adding retries is another change.
- These decisions are proposals: the owner reviews this change before any phase starts.

## Scope / non-goals

- A Google Calendar event that does not deserialize keeps failing its page: making it an entry needs io-gcal to keep what a page misses. Left for an io-gcal change.
- The active changes event-alarm-flag and jmap-backend are untouched.
- No version bump; no release.

## Order and parallel work

This change runs in parallel with cardamum's (cardamum/cairn/changes/a-result-carries-its-errors), ortie's and carillon's: they touch other repositories. The two shared inputs are written once elsewhere: io-pim-discovery's stream-0-4 (Phase 0) and io-webdav's a-listing-returns-what-it-could-not-read (before Phase 4). Both apps copy neverest's readers, never each other's half-done ones.
