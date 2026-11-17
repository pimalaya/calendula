---
cairn: log
change: anonymous-dav-and-punycode-hosts
date: 2026-11-17
---

# Anonymous DAV and Punycode hosts

Audit line COH-6, the owner's two rulings of 2026-11-17. Capabilities **config** and **packaging** moved.

- **src/config.rs**: `caldav.auth` defaults to `CaldavAuthConfig::None`, so a block with no `auth` loads and sends no `Authorization` header. `parse_url` maps url's `IdnaError` to "Host `<host>` is not ASCII: write it in its Punycode form (xn--...)"; `caldav.home` deserializes through it.
- **src/caldav/client.rs**: `parse_server` parses through `parse_url`, still refusing rather than prefixing a URL that fails. `unauthenticated` puts "CalDAV server answered 401: it asks for credentials, and the account configures no `auth`" above a 401 when the block sends no credential, the code left `null`.
- **src/shared/client.rs, src/session.rs, src/cli.rs, src/account/check.rs**: the sentence reaches the unified commands (`CalendarClient::explain`, applied by `run_unified`), the `caldav` namespace and `account check`.
- **Cargo.toml**: the `caldav` feature drops `url/serde`, nothing deserializing a `Url` any more.
- **Evidence**: `a_caldav_block_without_auth_loads` (src/config.rs), `a_non_ascii_host_names_its_punycode_form` (src/caldav/client.rs) and `a_401_without_auth_names_the_missing_auth` (src/account/check.rs, a loopback server answering 401) fail on the HEAD sources ("missing field `auth`", "invalid international domain name", and, with `auth = "none"` written so the old code loads the block, an `error` of `WebDAV server returned HTTP 401: Unauthorized` alone) and pass after.

CHANGELOG (Changed). config.sample.toml: `auth = "none"` is the same as no `auth`. Spec: config ("A CalDAV server may ask for no credentials", added, two scenarios), packaging ("Hosts are ASCII", "Dependencies name their features", modified).
