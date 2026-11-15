---
cairn: change
id: every-feature-set-builds-clean
status: landed
created: 2026-11-12
---

# Every feature set builds clean

## Why

Change 7 of the-family-standards. Himalaya (2026-11-11-every-feature-set-builds-clean, its CI `features` job since 53c17c98) and Cardamum (2026-11-12-every-feature-set-builds-clean) keep `cargo clippy --all-targets -- -D warnings` clean over their whole feature matrix and check it in CI; calendula checks nothing but the default build through the shared workflow.

Of calendula's sets, the default build, `--all-features`, each backend alone with `rustls-ring`, `rustls-aws,caldav` and `rustls-ring,vdir,pimdir` pass today. Two do not. The build with no backend (`--no-default-features --features rustls-ring`) fails on 27 errors in the binary and 29 with its tests: the unified client's arguments no arm reads and its two `Unused` stand-ins never built, `account check` and `account list` locals no backend pushes to, `BackendCheck::from` never called, the query's `range` and `kind` read by the backends alone, `PathBuf` imported for the backend blocks, and in the tests a `home` helper and a sample account only the backend tests read and a one-element loop over the backends. `native-tls,caldav` does not build in the devshell at all, which carries no OpenSSL (Himalaya's and Cardamum's shell.nix do), so dependencies-trimmed ran it with `PKG_CONFIG_PATH` on nixpkgs' openssl.dev.

cairn/spec/packaging.md and CONTRIBUTING.md call the build with no backend unsupported, yet it compiles and runs: `account list`, `completion`, `manual` and `json-schema` work, and `account check` and the unified commands fail, no backend matching being configured. Cardamum keeps that build in its matrix.

## What

- The build with no backend compiles clippy-clean. An item only the backends use is gated on any of them (`BackendCheck::from` and its `code_of` import, config's `PathBuf` import, the tests' `home`). A lint it raises on an item every build keeps is allowed on that item in that build alone (`cfg_attr(not(any(<every backend>)), allow(..))`, the any written out at each site): the query's `range` and `kind`, the `account check` and `account list` locals, the sample test's account, the backends' round-trip loop; the unified client's unread arguments once on its module, as Cardamum does. The `Unused` stand-ins, which exist only there, take a plain `allow(dead_code)`.
- No configuration type moves: the backend blocks stay compiled with their backend alone, a build ignoring the block of a backend it leaves out, as "Unknown fields" (config) says and as it does today. calendula has none of the cross-backend configuration types (proxy, SASL) Himalaya allows unread.
- shell.nix: openssl beside pkg-config, as Himalaya's, so `native-tls` builds without `vendored`.
- .github/workflows/tests.yml: a `features` job, Himalaya's and Cardamum's, running clippy `-D warnings` and `cargo test` over the matrix.
- Spec: "One cargo feature per backend" (packaging) lets a build compile without a backend; "Every backend builds alone" (packaging) added, naming the matrix. CONTRIBUTING.md stops calling that build unsupported.

## Scope / non-goals

No behaviour changes: every command, output and configuration a build accepted it still accepts. No CHANGELOG entry, nothing user-visible moving. The CONTRIBUTING feature table, stale on gcal, msgraph and the default set, is release-polish's.
