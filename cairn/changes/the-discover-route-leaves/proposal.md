---
cairn: change
id: the-discover-route-leaves
status: landed
created: 2026-11-15
---

# The discover route leaves

> Owner decision of 2026-11-15: calendula no longer depends on io-pim-discovery, as Himalaya, Ortie and neverest do not. cardamum drops `carddav.discover` the same day.

## Why

`caldav.discover` resolved a bare domain to a CalDAV root through RFC 6764 SRV records and the `.well-known` path, on every run. It was the one reason calendula linked io-pim-discovery and, under it, a DNS client (domain, resolv-conf, psl, ipconfig and windows-registry on Windows). The other routes already reach every provider calendula documents: `caldav.server` names the root, `caldav.home` the home set, and the providers publish both (Fastmail's `https://caldav.fastmail.com/dav/`, Google's `https://apidata.googleusercontent.com/caldav/v2/`, which no `.well-known` path leads to anyway).

## What

- `caldav.discover` is refused when the configuration loads, by name, as the round's other removed keys are (`calendar.list`, `event.list.table`): the message says the key is removed, to name the server in `caldav.server` (with the Fastmail and Google roots as examples) and points at MIGRATION.md. A build without `caldav` still loads the block unread.
- The CalDAV client connects from `server` or `home` alone; a block with neither fails naming those two.
- io-pim-discovery leaves Cargo.toml: the dependency, its `caldav` feature wiring, its forwarding from the TLS features and its `[patch.crates-io]` git pin, the last patch, so the section goes. Cargo.lock drops it and the crates only it pulled, nothing else moving.
- `caldav discover`, the command printing the principal and the home set, stays: it walks from the configured route through io-webdav and never used io-pim-discovery. Its help no longer claims an RFC 6764 step.
- config.sample.toml, README, CONTRIBUTING, the src/main.rs header, CHANGELOG (**Breaking**), MIGRATION.md.
- Spec: config ("CalDAV locates its home-set two ways"), backends ("CalDAV backend"), packaging ("One cargo feature per backend", "Dependency checks pass clean").
