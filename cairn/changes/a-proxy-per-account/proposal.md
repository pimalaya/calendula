---
cairn: change
id: a-proxy-per-account
status: landed
created: 2026-11-15
---

# A proxy per account

> Line K12 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), finding F40: cardamum's `proxy` keys, which ortie, calendula and neverest lacked.

## Why

calendula connected CalDAV and Graph with `Proxy::None` and Google Calendar through io-gcal's own connect, which reads the environment: no configuration could send a network backend through a SOCKS5 or HTTP proxy, and CalDAV and Graph ignored `all_proxy` too. cardamum has `proxy` at the account and under each network backend.

## What

- `proxy.url` (`socks5://`, `socks5h://` or `http://`), `proxy.username` and `proxy.password` (a secret) at the account and under `caldav`, `gcal` and `msgraph`, as cardamum spells them; a backend's own wins, the account's fills the others; with neither, the connection reads `all_proxy` and `https_proxy`, honouring `no_proxy` (pimalaya-stream's `Proxy::System`). A password without a username is refused. Additive keys: nothing breaks, but CalDAV and Graph now honour the proxy variables of the environment as Google already did.
- The account `proxy` loads unread in a build with no network backend.
- io-gcal's connect takes no proxy, so calendula opens the Google Calendar stream itself (pimalaya-stream's `connect_tls` on io-gcal's base URL, the same 30-second read timeout) and hands it to `GcalClientStd::new`.
- config.sample.toml carries the keys, commented, as cardamum's does.
- Spec: config.
