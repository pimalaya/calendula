---
cairn: log
change: a-check-probes-the-pinned-home
landed: 2026-11-15
---

# A check probes the pinned home

`account check` on a CalDAV account pinning `caldav.home` sends one PROPFIND Depth 0 on the home-set, so a credential the server refuses fails the row as it does on `caldav.server`. Line K1 of the CLI alignment round, twin of cardamum's C1. Capability **backends** moved.

- **src/account/check.rs**: `check_caldav`, on `home` alone, runs io-webdav's `WebdavPropfind` (Depth 0, `DAV:resourcetype`) on the home's path through the connected client's stream, its failure converted to `WebdavClientStdError` so the row reads and codes it as discovery's refusal. `caldav::client::connect` is unchanged: the unified commands on `home` still send nothing before their own first request. The help names the PROPFIND.
- **Evidence**: `account::check::caldav_tests` runs `check_caldav` against a local HTTP server. Answering 401, the `server` row and the `home` row fail alike, `code` `null` and `WebDAV server returned HTTP 401: Unauthorized`, the `home` route on one `PROPFIND /calendars/alice/` carrying `Depth: 0`; answering a 207 multistatus, the `home` row passes. Both failed with the probe skipped (`ok: true`, no request sent). No live test ran.

CHANGELOG (Fixed). Spec: backends ("account check authenticates on a pinned CalDAV home").
