---
cairn: delta
change: every-feature-set-builds-clean
---

# Delta

## ADDED Requirements

### Requirement: Every backend builds alone
(packaging) `cargo clippy --all-targets -- -D warnings` and `cargo test` SHALL pass with the default features; with `--all-features`; with `--no-default-features` and a TLS provider alone (`rustls-ring`), the build with no backend; with `rustls-ring` and each backend feature alone (`caldav`, `gcal`, `msgraph`, `vdir`, `pimdir`); with the other TLS providers beside CalDAV (`native-tls,caldav`, `rustls-aws,caldav`); and with the two local backends together (`rustls-ring,vdir,pimdir`), checked in CI by the `features` job of .github/workflows/tests.yml. The devshell SHALL carry OpenSSL, so `native-tls` builds without `vendored`. Code only some backends use SHALL be gated on the item with exactly those features, never allowed crate-wide; a lint only the build with no backend raises on an item every build keeps (an argument no arm reads, a stand-in variant never built) SHALL be allowed on that item, or once on the module, in that build alone. A backend's configuration block SHALL stay compiled with its backend alone, as "Unknown fields" (config) says, so a build ignores the block of a backend it leaves out.

#### Scenario: The build with no backend
- GIVEN `--no-default-features --features rustls-ring`
- WHEN `cargo clippy --all-targets -- -D warnings` runs
- THEN it passes, where it reported the unified client's arguments unread and its stand-ins never built

#### Scenario: A block for a compiled-out backend
- GIVEN a vdir-only build and an account whose `caldav` block carries an unknown key
- WHEN `account list` reads the configuration
- THEN the account is listed with vdir alone, where a build with `caldav` refuses the key

## MODIFIED Requirements

### Requirement: One cargo feature per backend
(packaging) Each backend SHALL sit behind its own cargo feature (`caldav`, `gcal`, `msgraph`, `vdir`, `pimdir`), pulling its io-* crates through `dep:`. A build with none SHALL compile: it serves `account list`, `completion`, `manual` and `json-schema`, and `account check` and the unified commands fail, no backend matching being configured.

`caldav` SHALL pull io-pim-discovery, with its RFC 6764 resolution alone, for the `discover` route, so a build without that backend links no discovery code.

`gcal` SHALL additionally pull the time zone database its projection mints definitions from, so a build without that backend pays neither the dependency nor its embedded data.

## REMOVED Requirements

None.
