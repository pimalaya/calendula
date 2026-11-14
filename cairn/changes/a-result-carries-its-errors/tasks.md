---
cairn: tasks
change: a-result-carries-its-errors
---

# Tasks

## Working rules

- Read first: this change's proposal.md and delta.md; himalaya/cairn/changes/a-result-carries-its-errors/{proposal,delta}.md and himalaya/cairn/log/2026-11-14-a-result-carries-its-errors.md; neverest/cairn/changes/a-refusal-carries-its-code/proposal.md; neverest/src/error.rs (`Failure`, `code`, `refusal_code`) and its readers (`refusal_code` in neverest/src/dav/client.rs, src/gcal/client.rs, src/msgraph/client.rs; `google_code` in src/throttle.rs); himalaya/src/error.rs (`ItemError` alias); pimalaya-cli's `error::ItemError` (cli/src/error.rs); calendula's CONTRIBUTING.md and the src/main.rs header.
- Every cargo command runs as `nix develop --command cargo ...`. `cargo fmt` after each phase.
- An agent edits, builds and tests; it runs no git write command. The session owning the repository commits each phase with the message given here, no AI trailer.
- Never adjust production code to fit a test. A regression test must fail against the code before the phase.
- No em dashes; code comments wrap at 80 columns; markdown is never hard-wrapped.
- Phase checks, each one: `cargo test --all-features`, then the CI `features` job's loop (CONTRIBUTING.md, .github/workflows/tests.yml): `cargo clippy --locked --all-targets -- -D warnings` and `cargo test --locked` over the default set, `--all-features`, `--no-default-features --features rustls-ring`, `native-tls,caldav`, `rustls-aws,caldav`, `rustls-ring,vdir,pimdir`, and `rustls-ring,<b>` for each backend alone; `sh cairn/verify.sh`.

## Phase 0, outside this repository

- [x] io-pim-discovery's stream-0-4 is committed and pushed; note its commit hash for Phase 1. Stop here until it is. Committed locally as 1c3d195 (version 0.12.0), not pushed: Phase 1 links it by a local path patch (owner ruling).

## Phase 1, the libraries (commit `build: update pimalaya-stream to 0.4 and the io-* crates`)

- [x] Cargo.toml: `pimalaya-stream` 0.4, `io-http` 0.8, `io-msgraph` 0.9, dev `io-oauth` 0.7.
- [x] `[patch.crates-io]`, sorted, no comment: io-gcal dbb4481cd77ef65ca451420cb4e087fdccfe6fdb, io-pim-discovery (Phase 0 rev), io-webdav 185ab63be5859c5ef6115bef97e4df5cb1187973, pimalaya-cli 0be1b1f08d19ccb9d16ee8e1d4266c27c96e0cdd. A crate the owner released meanwhile is taken from crates.io, its pin dropped. Done with io-pim-discovery as `{ path = "../io-pim-discovery" }` (1c3d195 is not pushed; owner ruling); none of the four was released on crates.io.
- [x] `cargo update`; the lock holds pimalaya-config 0.4.1.
- [x] src/config.rs:546-561: `server: self.cert.map(|cert| TlsServer { cert }), client: None`.
- [x] `cargo tree -d --all-features -e normal`: no pimalaya-stream, io-http or io-oauth twice.
- [x] tests/google.rs and tests/microsoft.rs build against io-oauth 0.7 (`Tls::default()`).
- [x] CHANGELOG: the dependency entry (Changed, about :67) follows; Fixed: a prompting credential command no longer hangs (pimalaya-config 0.4.1).
- [x] Phase checks pass.

## Phase 2, the codes (commit `feat: code a server's refusal and a missing item or calendar`)

- [ ] src/error.rs: `PermissionDenied`, `QuotaExceeded`, `RateLimited`, `ItemNotFound`, `CalendarNotFound`, `ItemUnreadable` (and `CalendarUnreadable` only if Phase 3 meets it), declared in every build (`allow(dead_code)` gated as neverest gates it); `error::code` and the chain walk over one reader per enabled backend; the `ItemError` alias.
- [ ] CalDAV reader (src/caldav/): 507 or `quota-not-exceeded`, 429, 403 with `need-privileges` or no `DAV:error` body, `DuplicateUid` `item-duplicate`, 412 `precondition-failed`, 404/410 on an item request.
- [ ] Google Calendar reader (src/gcal/) with a copy of neverest's `google_code` and the `google_refusal` parts it reads; 412; 404/410 on an event request.
- [ ] Graph reader (src/msgraph/): `ErrorQuotaExceeded` or 507, 429, 403, 412, 404 or 400 `ErrorInvalidIdMalformed` on an event request; the `--if-match` mismatch `bail!`s (src/msgraph/backend.rs:250-253, :276-279) become `precondition-failed`.
- [ ] vdir (src/vdir/backend.rs:91) and pimdir (src/pimdir/backend.rs:603, :530) not-found sites coded; src/shared/event/delete.rs:123 `item-not-found`.
- [ ] src/shared/event/delete.rs:91: the lookup falls back to a full listing on `item-not-found` only.
- [ ] src/account/check.rs:254: rows coded through `error::code`.
- [ ] Tests through the real error path, failing before the phase: Graph and Google Calendar over their scripted streams (src/msgraph/backend.rs:410-440 builds raw responses with any status; src/gcal/backend.rs:565-612's `Scripted` takes any raw response: add a helper answering a status other than 200); CalDAV through a `WebdavClientStd` over a scripted stream (none exists: build one beside the Graph one, a 403, a 507, a 412 and a `no-uid-conflict` body); `event delete` over a scripted Graph answering 403 to the lookup fails coded and lists nothing; `item read` of an unknown id over a temp vdir prints `item-not-found` and exits 1.
- [ ] Spec: "A JSON error carries a stable code" and "A server's refusal carries its code" (see delta.md).
- [ ] Phase checks pass.

## Phase 3, the listings (commit `feat!: list the calendars and items a listing could not read`)

- [ ] `CalendarListOutput` (src/shared/calendar/list.rs:46), `EventListOutput` (src/shared/event/list.rs:162, also `event find`), `ItemListOutput` (src/shared/item/list.rs:101) gain `errors: Vec<ItemError>`, required in their schema; `EventAgendaOutput` (agenda.rs:1030) serializes `{starts, errors}` (its custom `Serialize` :1088 and `JsonSchema` :1107 follow); the unified client (src/shared/client.rs:445-470) carries the entries beside the items.
- [ ] An unparseable item in `event list`, `event find` and the agenda: an `item-unreadable` entry (src/shared/event/expand.rs:74-77, :107-110 return the error instead of nothing, or their callers keep it); `event list`'s `count` counts listed events (list.rs:139).
- [ ] vdir (src/vdir/backend.rs:115-131): `list_readable_items`, one entry per failed file; a vanished file left out silently. Check the collection listing on an unreadable metadata file: an entry coded `calendar-unreadable` if it fails today, else drop that code.
- [ ] Graph (src/msgraph/backend.rs:184-189): a refused re-read is an entry, a 404 leaves the event out, the others listed.
- [ ] pimdir (src/pimdir/backend.rs:611-625, :613, :1094-1097): entries for an unreadable blob, a blob the tree does not hold and a hydrated body that does not parse; the `warn!` goes; unhydrated items unchanged.
- [ ] Paging: whole-read backends print their entries on the first page only; pimdir on the page holding the item.
- [ ] A non-empty `errors` returns `PartialOutcome::default()` after printing (exit 3).
- [ ] Text: src/shared/listing.rs moves the paging fields from the footer into the header (`gcal instances` included). Every listing prints its `Error:` lines last, after its rows and one blank line, nothing at all when `errors` is empty. The text tests follow the moved paging and gain a case with an entry; "Text output is plain text" folded as delta.md words it.
- [ ] json-schema: `calendula-item-error` registered (src/json_schema.rs:206).
- [ ] MCP: the instructions (src/mcp.rs:62-64) name `errors`; tests/mcp.rs gains an `event_list` over a vdir holding an unparseable item, asserting `isError: true` with `structuredContent.errors`.
- [ ] `Session::run_unified` (src/session.rs:44-66) keeps its client after a `PartialOutcome`; tests/repl.rs asserts a second line reuses the session after an exit-3 line.
- [ ] Tests: tests/event_list.rs over a temp vdir holding one unparseable `.ics` and one `chmod 000` file (unix only) prints the others, two entries, exit 3; the agenda's JSON root; tests/usage.rs or a new tests/exit.rs case for exit 3; the live suites' helpers (tests/common.rs `assert_events`) adjusted if they compare whole objects.
- [ ] Spec: the listing requirements folded (see delta.md).
- [ ] Phase checks pass.

## Phase 4, CalDAV rows (after io-webdav's a-listing-returns-what-it-could-not-read is pushed and pinned)

- [ ] Pin io-webdav to that commit; src/caldav/backend.rs:112-125 reads the new list output.
- [ ] Each unread row an `item-unreadable` entry coded from its status; a truncated listing an entry with a `null` id.
- [ ] A scripted multistatus test with a 403 row and a 507 row.
- [ ] Spec: "CalDAV backend" says it. Commit `feat: list the CalDAV rows a listing could not read`.

## Landing

- [ ] CHANGELOG (Added, **BREAKING**, Changed, Fixed as in the proposal), MIGRATION.md ("JSON output"), README exit statuses, the src/main.rs header.
- [ ] Delta folded into cairn/spec/commands.md, backends.md and mcp.md; log cairn/log/<date>-a-result-carries-its-errors.md (frontmatter `cairn: log`, `change`, `landed:`) naming what landed per phase, the evidence and the capabilities moved; status landed; `sh cairn/verify.sh` clean.
- [ ] The live suites (google, microsoft, fastmail) run by the owner on the result.
