---
cairn: tasks
change: the-contract-audit-rulings
---

# Tasks

- [x] Cargo.toml: `[patch.crates-io] pimalaya-cli = { path = "../cli" }`, to become a release pin before commit.
- [x] src/config.rs, config.sample.toml: TLS `client`, account `tls`, `inherit_tls`.
- [x] src/account/check.rs: the list of accounts, `tls` on network rows.
- [x] src/main.rs, src/cli.rs: bare invocation through the kit; `protocol_exit`; `accounts` alias.
- [x] src/capability.rs: unified names, `event.search`.
- [x] src/pimdir/queue.rs, queue/list.rs, queue/show.rs, queue/cancel.rs, status.rs: as cardamum's.
- [x] src/pimdir/search.rs: `search_page`, `--page-token` text cursor.
- [x] src/caldav/delete.rs, rmcol.rs, cli.rs: item delete, `rmcol`.
- [x] src/shared/event/read.rs, search.rs (find.rs removed), cli.rs, client.rs, mcp.rs: `--uid`, the query search.
- [x] src/shared/event.rs, expand.rs, src/pimdir/index.rs: `calendarId`, empty ids as `null`.
- [x] src/error.rs, src/repl.rs, src/shared/ical.rs, src/shared/client.rs, src/pimdir/client.rs, backend.rs, vdir/list.rs, account/check.rs: codes, wording, layouts.
- [x] Tests, CHANGELOG, spec fold, log.
