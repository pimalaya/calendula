---
cairn: delta
change: dependencies-trimmed
---

# Delta

## ADDED Requirements

### Requirement: Dependencies name their features
(packaging) Every dependency, build and dev ones included, SHALL be declared with `default-features = false` and the features calendula uses, each checked against the item it gates, so a default turned on upstream never reaches the binary. A feature only one backend needs SHALL be turned on by that backend's cargo feature: url's `serde`, which only `caldav.home` deserializes through, by `caldav`. A dependency nothing uses SHALL be removed, and a crate the tests alone use SHALL be a dev-dependency.

### Requirement: Hosts are ASCII
(packaging) calendula SHALL build url's IDNA without a Unicode back end, `idna_adapter` being pinned to 1.0.0 in Cargo.toml. A host SHALL be written in ASCII, an internationalised one in its Punycode form (`xn--…`), and a CalDAV `server` or `home` with a non-ASCII host SHALL be refused when parsed: `home` when the configuration loads, `server` when the account connects. The Google Calendar and Microsoft Graph hosts are fixed ASCII names.

#### Scenario: An internationalised CalDAV server
- GIVEN `caldav.server = "https://bücher.example/dav/"`
- WHEN the account connects
- THEN the server is refused before any connection, and the same account connects with `caldav.server = "https://xn--bcher-kva.example/dav/"`

### Requirement: Dependency checks pass clean
(packaging) `cargo deny check` SHALL pass its advisories, bans, licences and sources checks over the targets deny.toml lists, which are the release targets and the ones people build from source, allowing only the licences the tree carries. A duplicate the tree cannot avoid stays a visible warning rather than a skip: today getrandom 0.2 (ring 0.17) and syn 2 (derive_more-impl under inquire's crossterm, domain-macros under io-pim-discovery, windows-core under chrono on Windows).

## MODIFIED Requirements

### Requirement: CalDAV locates its home-set three ways
(config) The `caldav` block SHALL offer exactly three mutually exclusive routes, from most to least discovery. `discover` resolves a bare domain through RFC 6764 SRV records and the `.well-known` path.

`server` names the context root the principal and home-set walk starts from, taking a full URL, a bare domain or `domain:port` with `https` assumed for a bare authority, and is parsed when it is used rather than when it is read. A string that fails to parse as a URL for any reason other than lacking a scheme SHALL be refused rather than read as a bare authority. `home` pins the home-set outright.

A block carrying none of the three SHALL be rejected by name.

## REMOVED Requirements

None.
