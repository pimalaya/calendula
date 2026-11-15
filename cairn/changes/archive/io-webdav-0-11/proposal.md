---
cairn: change
id: io-webdav-0-11
status: landed
created: 2026-11-15
---

# io-webdav 0.11

> calendula's part of lines K27 and W10 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), after io-webdav 0.11.0 (W9).

## Why

- **K27.** Two CalDAV requests were pumped by hand over the client's stream: the REPORT of `caldav report raw` (src/caldav/report/raw.rs `run_report`) and the PROPFIND Depth 0 `account check` sends on a pinned `caldav.home` (src/account/check.rs `check_caldav`). Neither went through io-webdav's resend, so a server that closed the connection before answering (`Connection: close`, an idle keep-alive timing out) failed them with `reached unexpected EOF` or `connection reset`, where every other CalDAV request is sent again on a fresh connection. io-webdav 0.11 makes `WebdavClientStd::run` public for a coroutine the client has no method for.
- The raw REPORT's error reached the chain as io-webdav's `WebdavSendError` alone, which calendula's CalDAV refusal reader (`refusal_code`, reading `WebdavClientStdError`) never coded: a 403 failed uncoded, against backends "A server's refusal carries its code".
- **W10.** io-webdav 0.11 adds `href` beside every listed, enumerated, multiget and written id. calendula keys nothing on DAV ids and keeps the decoded ids.

## What

- io-webdav 0.11.0 (`cargo update -p io-webdav`, the lock diff that crate alone), the requirement raised to `0.11.0`. Both `CaldavCalendar` literals (src/caldav/backend.rs, src/caldav/mkcalendar.rs) already end in `..Default::default()`, so `href` needs no code.
- src/caldav/report/raw.rs: the REPORT runs through `client.run`; `run_report` goes. Its output is unchanged; a refusal now carries the code any CalDAV refusal does.
- src/account/check.rs: the PROPFIND runs through `client.run`, its row's code and error unchanged.
- Spec: backends ("A CalDAV request survives a closed connection", added).

## Scope / non-goals

- No version bump: the release is the owner's.
