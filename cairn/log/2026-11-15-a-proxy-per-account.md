---
cairn: log
change: a-proxy-per-account
landed: 2026-11-15
---

# A proxy per account

`proxy.url`, `proxy.username` and `proxy.password` at the account and under `caldav`, `gcal` and `msgraph`, as cardamum spells them, line K12 of the CLI alignment round. Capability **config** moved.

- **src/config.rs**: `ProxyConfig`, handed from the account to every network backend naming none; with neither, `Proxy::System` reads `all_proxy` and `https_proxy`, where CalDAV and Graph connected with `Proxy::None`.
- **src/caldav/client.rs, src/msgraph/backend.rs, src/gcal/client.rs**: each connects through it; Google Calendar opens its stream itself, io-gcal's connect taking no proxy.
- **config.sample.toml**: the keys, commented.
- **Evidence**: `proxy_tests` in src/config.rs read, on a local HTTP proxy refusing every tunnel, the `CONNECT` each backend sends; the code before refused the `proxy` key.

CHANGELOG (Added). Spec: config ("A proxy is set per account and per backend", added; "Backend blocks", "Unknown fields").
