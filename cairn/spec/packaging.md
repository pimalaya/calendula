---
cairn: spec
capability: packaging
status: current
---

# Packaging

### Requirement: calendula is an application, not a library
The crate SHALL ship a binary only, with no library target. Its architecture document is the src/main.rs header, not a separate file, and the manifest carries no `documentation` field and no docs.rs metadata.

### Requirement: One cargo feature per backend
Each backend SHALL sit behind its own cargo feature (`caldav`, `gcal`, `msgraph`, `vdir`, `pimdir`), pulling its io-* crates through `dep:`. A build with none SHALL compile: it serves `account list`, `completion`, `manual` and `json-schema`, and `account check` and the unified commands fail, no backend matching being configured.

`caldav` SHALL pull io-pim-discovery, with its RFC 6764 resolution alone, for the `discover` route, so a build without that backend links no discovery code.

`gcal` SHALL additionally pull the time zone database its projection mints definitions from, so a build without that backend pays neither the dependency nor its embedded data.

### Requirement: TLS providers are orthogonal
`rustls-ring` (the default), `rustls-aws` and `native-tls` SHALL each select a pimalaya-stream provider and forward it to every network dependency, weakly where that dependency is optional. `vendored` SHALL forward the same way.

### Requirement: Released dependencies, patched only while unreleased
Dependencies SHALL name released versions. A `[patch.crates-io]` entry is allowed only while a needed change rides unreleased upstream, the cairn change adding it SHALL say what it waits on (Cargo.toml carrying no comment in its dependency or patch sections), and it SHALL be dropped as that crate publishes.

### Requirement: No aggregator dependency
calendula SHALL depend on the protocol crates directly and own the abstraction over them. The retired per-domain aggregators (io-calendar and its siblings) SHALL NOT be reintroduced.

### Requirement: The sample ships beside the binary
The Nix package SHALL install config.sample.toml as share/doc/calendula/config.sample.toml, the release archives SHALL carry it with the share/ of the build they come from, and install.sh SHALL copy it to the same path under its prefix when the downloaded archive holds it, so the first-run welcome copies it rather than downloading it.

### Requirement: Dependencies name their features
Every dependency, build and dev ones included, SHALL be declared with `default-features = false` and the features calendula uses, each checked against the item it gates, so a default turned on upstream never reaches the binary. A feature only one backend needs SHALL be turned on by that backend's cargo feature: url's `serde`, which only `caldav.home` deserializes through, by `caldav`. A dependency nothing uses SHALL be removed, and a crate the tests alone use SHALL be a dev-dependency.

### Requirement: Hosts are ASCII
calendula SHALL build url's IDNA without a Unicode back end, `idna_adapter` being pinned to 1.0.0 in Cargo.toml. A host SHALL be written in ASCII, an internationalised one in its Punycode form (`xn--…`), and a CalDAV `server` or `home` with a non-ASCII host SHALL be refused when parsed: `home` when the configuration loads, `server` when the account connects. The Google Calendar and Microsoft Graph hosts are fixed ASCII names.

#### Scenario: An internationalised CalDAV server
- GIVEN `caldav.server = "https://bücher.example/dav/"`
- WHEN the account connects
- THEN the server is refused before any connection, and the same account connects with `caldav.server = "https://xn--bcher-kva.example/dav/"`

### Requirement: Dependency checks pass clean
`cargo deny check` SHALL pass its advisories, bans, licences and sources checks over the targets deny.toml lists, which are the release targets and the ones people build from source, allowing only the licences the tree carries. A duplicate the tree cannot avoid stays a visible warning rather than a skip: today getrandom 0.2 (ring 0.17) and syn 2 (derive_more-impl under inquire's crossterm, domain-macros under io-pim-discovery, windows-core under chrono on Windows).

### Requirement: Every backend builds alone
`cargo clippy --all-targets -- -D warnings` and `cargo test` SHALL pass with the default features; with `--all-features`; with `--no-default-features` and a TLS provider alone (`rustls-ring`), the build with no backend; with `rustls-ring` and each backend feature alone (`caldav`, `gcal`, `msgraph`, `vdir`, `pimdir`); with the other TLS providers beside CalDAV (`native-tls,caldav`, `rustls-aws,caldav`); and with the two local backends together (`rustls-ring,vdir,pimdir`), checked in CI by the `features` job of .github/workflows/tests.yml. The devshell SHALL carry OpenSSL, so `native-tls` builds without `vendored`. Code only some backends use SHALL be gated on the item with exactly those features, never allowed crate-wide; a lint only the build with no backend raises on an item every build keeps (an argument no arm reads, a stand-in variant never built) SHALL be allowed on that item, or once on the module, in that build alone. A backend's configuration block SHALL stay compiled with its backend alone, as "Unknown fields" (config) says, so a build ignores the block of a backend it leaves out.

#### Scenario: The build with no backend
- GIVEN `--no-default-features --features rustls-ring`
- WHEN `cargo clippy --all-targets -- -D warnings` runs
- THEN it passes, where it reported the unified client's arguments unread and its stand-ins never built

#### Scenario: A block for a compiled-out backend
- GIVEN a vdir-only build and an account whose `caldav` block carries an unknown key
- WHEN `account list` reads the configuration
- THEN the account is listed with vdir alone, where a build with `caldav` refuses the key

### Requirement: Live provider suites
The binary SHALL carry one ignored integration suite per provider under tests/ (google.rs for the Google Calendar API and Google CalDAV, microsoft.rs for Microsoft Graph, fastmail.rs for Fastmail CalDAV), run with `--ignored` and sharing tests/common.rs. Each SHALL hold an `anonymous` module that runs every backend the provider's sample sections offer on a made-up credential and touches no account, its accounts taken from the provider sections of config.sample.toml so a provider drifting from the sample fails it, and asserting the exit code of the output contract (3 with the `account check` report, 1 with the failure object) and a message naming the refusal in calendula's words (`HTTP 401`). It MAY hold a `credentialed` module that mints its own token from `CALENDULA_`-prefixed variables and only reads (`account check`, `calendar list`, `event list`), failing naming the variables it misses, which CI runs in the upstream repository only and once its secrets exist; Google's SHALL ask `calendar.events` and `calendar.calendarlist.readonly` alone, and so covers the Google Calendar API only. A token or password SHALL reach the binary through a command reading its environment, never argv nor the configuration, and no assertion SHALL print a token, a secret or an event's value. CI SHALL run each suite in its own job at `RUST_LOG=debug` at most.
