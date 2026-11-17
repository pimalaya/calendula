---
cairn: change
id: anonymous-dav-and-punycode-hosts
status: landed
created: 2026-11-17
---

# Anonymous DAV and Punycode hosts

## Why

Audit line COH-6 of the CLI family analysis (2026-11-15) and two owner rulings of 2026-11-17. A `caldav` block with no `auth` loaded in carillon and failed "missing field `auth`" in calendula, cardamum and neverest; a non-ASCII host was refused with url's bare "invalid international domain name". The rulings: `auth` is optional everywhere, absent sending no `Authorization` header, a 401 then saying the account configures no `auth`; a non-ASCII host is refused everywhere, the message pointing at its Punycode form.

## What

- `caldav.auth` optional (`#[serde(default)]`, `CaldavAuthConfig::None` the default), `auth = "none"` still accepted.
- A 401 to a block with no `auth` fails with "CalDAV server answered 401: it asks for credentials, and the account configures no `auth`", its code unchanged (`null`), on the unified commands, the `caldav` namespace and `account check`.
- A CalDAV `server` or `home` whose host is not ASCII is refused with "Host `<host>` is not ASCII: write it in its Punycode form (xn--...)".
- `caldav.home` is read as a string and parsed by calendula, so the `caldav` feature no longer turns on url's `serde`.
