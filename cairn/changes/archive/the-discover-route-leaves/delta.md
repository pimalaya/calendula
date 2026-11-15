---
cairn: delta
change: the-discover-route-leaves
---

# Delta

## ADDED Requirements

## MODIFIED Requirements

### Requirement: CalDAV locates its home-set two ways
(config) Renamed from "CalDAV locates its home-set three ways". The `caldav` block SHALL offer exactly two mutually exclusive routes, `server` and `home`, and calendula SHALL resolve no domain through DNS. A block carrying neither SHALL be rejected by name.

A `caldav.discover` key, the RFC 6764 route an earlier calendula offered, SHALL fail the load by name in a build with the `caldav` feature (one without it loads the block unread), as the family refuses its removed keys, the message naming `caldav.server` with a provider root as example and MIGRATION.md, rather than as an unknown field.

#### Scenario: A discover route left in a configuration
- GIVEN a configuration written for 0.2 with `caldav.discover = "fastmail.com"`
- WHEN any calendula command loads it
- THEN it fails before reaching a backend, naming the line, saying the `caldav.discover` key is removed and to set `caldav.server`, and pointing at MIGRATION.md

### Requirement: CalDAV backend
(backends) The client SHALL connect from `caldav.server` or `caldav.home` alone, and SHALL NOT resolve a domain through DNS (RFC 6764 SRV records), so the backend links no resolver.

### Requirement: Dependency checks pass clean
(packaging) The syn 2 duplicate no longer comes from domain-macros under io-pim-discovery.

## REMOVED Requirements

### Requirement: One cargo feature per backend
(packaging) Only the paragraph "`caldav` SHALL pull io-pim-discovery, with its RFC 6764 resolution alone, for the `discover` route" goes; the requirement stays.
