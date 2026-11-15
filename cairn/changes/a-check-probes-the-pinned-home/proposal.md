---
cairn: change
id: a-check-probes-the-pinned-home
status: landed
created: 2026-11-15
---

# A check probes the pinned home

> Line K1 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), twin of cardamum's C1.

## Why

`account check` on an account whose CalDAV block pins `caldav.home` proved nothing about its credential: `caldav::client::connect` only opens the stream on that route (src/caldav/client.rs:67-73), so a password or token the server refuses reported `ok: true` and exit 0, where the `server` route's discovery fails the row. cardamum's CI went red on the same branch (`anonymous::carddav_account_check_refused`).

## What

- `check_caldav` (src/account/check.rs), on `home` alone, sends one PROPFIND Depth 0 asking `DAV:resourcetype` on the pinned home-set, through the stream the connected client holds; a failure reaches the row as a `WebdavClientStdError`, so it reads and codes as discovery's: `WebDAV server returned HTTP 401: <summary>`, `code` `null`, on both routes.
- The request belongs to the check, not to `connect`. Every unified command on `home` sends its own request on the home or below it first (a PROPFIND Depth 1 for `calendar list`, a REPORT, GET, PUT or DELETE for the others), which a refused credential already fails with the server's answer; a probe in `connect` would add a round-trip to every command, the one `home` exists to save. `account check` is the command whose whole job is to exercise the backend ("account check fails when a backend fails" in commands: it SHALL exercise every backend).
- Spec: backends ("account check authenticates on a pinned CalDAV home", added). The `account check` help names the PROPFIND.
