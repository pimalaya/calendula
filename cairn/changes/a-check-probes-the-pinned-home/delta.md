---
cairn: delta
change: a-check-probes-the-pinned-home
---

# Delta

## ADDED Requirements

### Requirement: account check authenticates on a pinned CalDAV home
(backends) `account check` on CalDAV SHALL send at least one authenticated request: on `server`, the principal and home-set discovery the client connects with; on `home`, which skips discovery, a PROPFIND Depth 0 on the pinned home-set asking its `DAV:resourcetype`. That PROPFIND SHALL be the check's alone: a unified command connecting on `home` sends nothing before its own first request, which a refused credential fails. A credential the server refuses SHALL fail the row on either route with the same code and error (`WebDAV server returned HTTP 401`, no code); a home answering anything but a multistatus fails it too.

#### Scenario: A refused credential on a pinned home
- GIVEN the account `test` whose `caldav.home` pins a home-set on a server answering 401 to its credential
- WHEN `calendula --json account check` runs
- THEN the `caldav` row reads `"ok": false`, `"code": null` and the error the same account gives on `caldav.server`, and the command exits 3

## MODIFIED Requirements

## REMOVED Requirements
