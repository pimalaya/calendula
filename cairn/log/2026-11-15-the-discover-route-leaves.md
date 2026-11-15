---
cairn: log
change: the-discover-route-leaves
landed: 2026-11-15
---

# The discover route leaves

The `discover` route of the `caldav` block is gone with the io-pim-discovery dependency (owner decision of 2026-11-15, as Himalaya, Ortie and neverest): calendula resolves no domain through DNS, `caldav.server` or `caldav.home` reaching the server. Capabilities **config**, **backends** and **packaging** moved.

- **Cargo.toml**: io-pim-discovery leaves the dependencies, the `caldav` feature and the TLS features' forwarding; its git pin was the last `[patch.crates-io]` entry, so the section goes. **Cargo.lock** drops io-pim-discovery, domain, domain-macros, octseq, psl, psl-types, resolv-conf, ipconfig, socket2, widestring and windows-registry, and moves nothing else.
- **src/config.rs**: `CaldavConfig::discover` is a `CaldavDiscoverRemovedConfig`, failing the load with "The `caldav.discover` key is removed, calendula resolving no domain: name the server in `caldav.server` instead (Fastmail's ..., Google's ...), as MIGRATION.md shows", the shape of the `calendar.list` and `event.list.table` refusals.
- **src/caldav/client.rs**: `connect` takes `home`, else `server`, else fails "CalDAV config needs `server` or `home`"; `resolve_server` and its default resolver go. **src/caldav/discover.rs**: `caldav discover` stays (it walks the principal and the home set through io-webdav), its help no longer naming an RFC 6764 step.
- **Evidence**: tests/account.rs `a_caldav_discover_key_is_refused_naming_server`, through the binary, under `--json` and in text; on the old code the same configuration loaded and `account list` exited 0.

config.sample.toml (its `discover` line is a `server` under a path), README (the RFC 6764 row, the `caldav` routes, the Google CalDAV note), CONTRIBUTING (the `caldav` feature row), the src/main.rs header, CHANGELOG (Removed, **Breaking**; the unreleased `caldav.discover` entries dropped), MIGRATION.md. Spec: config ("CalDAV locates its home-set two ways"), backends ("CalDAV backend"), packaging ("One cargo feature per backend", "Dependency checks pass clean").
