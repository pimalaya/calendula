---
cairn: log
change: a-caldav-server-is-read-as-written
landed: 2026-11-18
---

# A CalDAV server is read as written

Landed without a proposal: TWN-4 as cardamum landed it (48cd1a5), the owner's ruling on the `.well-known` probe relayed by cardamum's session, and CAL-18 of the deep analysis.

- **TWN-4**: `parse_server` reads a value with no `://` as `https://` followed by it, and one naming a scheme as written, refusing it when it does not parse or its scheme is not `http` nor `https`; the former `Ok(_) =>` arm turned `ftp://x` into the host `ftp`. A `server` naming an `https` origin is probed at `.well-known/caldav` through io-http's `Http11WellKnown` over a fresh TLS stream, and the walk starts from the redirect target when it is `https` (`https_only`), as cardamum does. Tests `a_scheme_other_than_http_is_refused`, `a_well_known_redirect_is_followed_over_https_only`.
- **CAL-18**: `connect` refuses a block naming both `server` and `home` (`a_block_naming_server_and_home_is_refused`), where `home` won silently against the spec's "mutually exclusive"; `pimdir search` reads a page size of 0 as every hit on one page, where it printed one per page (tests/pimdir_search.rs). CAL-18's third count, a missing blob read as not downloaded by `pimdir search` and as `item-unreadable` by the listing, closed with TRI-3 (1583fbb): both read it as not downloaded now.

Not run against a live server: the probe follows cardamum's, whose live cells ran on Fastmail.

Capabilities moved: config, commands.
