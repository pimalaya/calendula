---
cairn: tasks
change: the-discover-route-leaves
---

# Tasks

- [x] Cargo.toml: drop io-pim-discovery, its `dep:` in `caldav`, its `?/` forwarding in `native-tls`, `rustls-aws` and `rustls-ring`, and the `[patch.crates-io]` section it was the last entry of.
- [x] Cargo.lock: drop io-pim-discovery and the crates only it pulled (domain, domain-macros, octseq, psl, psl-types, resolv-conf, ipconfig, socket2, widestring, windows-registry), nothing else moving.
- [x] src/caldav/client.rs: `connect` takes `server` or `home`; `resolve_server` and `DEFAULT_RESOLVER` go.
- [x] src/config.rs: `CaldavConfig::discover` becomes `CaldavDiscoverRemovedConfig`, refusing the key by name.
- [x] src/caldav/discover.rs: the command's help drops the RFC 6764 step it never ran.
- [x] Offline test through the binary (tests/account.rs): `caldav.discover` fails the load under `--json` and in text, naming `caldav.server`, the Fastmail root and MIGRATION.md; it loaded on the old code.
- [x] config.sample.toml, README, CONTRIBUTING, the src/main.rs and src/config.rs headers, CHANGELOG, MIGRATION.md.
- [x] Fold the delta, log, set `landed`.
