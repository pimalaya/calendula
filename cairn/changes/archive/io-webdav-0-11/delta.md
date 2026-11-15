---
cairn: delta
change: io-webdav-0-11
---

# Delta

## ADDED Requirements

### Requirement: A CalDAV request survives a closed connection
Every CalDAV request calendula sends, a unified command's, a `caldav` command's (`report raw` included) and `account check`'s PROPFIND on a pinned home alike, SHALL go through io-webdav's client, by one of its methods or, for a request it has no method for, `WebdavClientStd::run`, never by hand over its stream: a request meeting a connection the server closed before answering it (`Connection: close`, an idle keep-alive timing out) is sent again, once, on a fresh connection to the same server. A refusal such a request meets SHALL carry the code any CalDAV refusal carries ("A server's refusal carries its code").

#### Scenario: A pinned home closing the check's connection
- GIVEN the account `test` whose `caldav.home` pins a home-set on a server closing the check's first connection unanswered, then answering its PROPFIND
- WHEN `calendula --json account check` runs
- THEN the PROPFIND is sent again on a fresh connection, the `caldav` row reads `"ok": true` and the command exits 0

#### Scenario: A raw REPORT refused
- GIVEN a CalDAV server answering a REPORT 403 with no precondition
- WHEN `calendula --json caldav report raw cal query.xml` runs
- THEN it exits 1 coded `permission-denied`

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
