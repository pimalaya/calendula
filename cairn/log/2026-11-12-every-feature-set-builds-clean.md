---
cairn: log
change: every-feature-set-builds-clean
landed: 2026-11-12
---

# Every feature set builds clean

`cargo clippy --all-targets -- -D warnings` and `cargo test` pass over the whole feature matrix, checked in CI as Himalaya's and Cardamum's are, change 7 of the-family-standards. Before, as clippy errors per set: the default features, `--all-features`, `rustls-ring` with each of `vdir`, `caldav`, `gcal`, `msgraph` and `pimdir`, `rustls-aws,caldav` and `rustls-ring,vdir,pimdir` none; `rustls-ring` alone, the build with no backend, 27 in the binary and 29 with its tests; `native-tls,caldav` no build at all in the devshell, openssl-sys finding no OpenSSL, and none once it did. After, none in every set.

- The binary's 27, all in the build with no backend: 20 arguments of the unified client's eight methods no arm reads, its two `Unused` stand-ins never built, `BackendCheck::from` never called, the query's `range` and `kind` never read, the `account check` and `account list` locals `mut` for nothing, and config's `PathBuf` import. The tests add the `home` helper of src/config.rs, the sample test's account and the one-element loop over the backends.
- Gated on any backend (`any(feature = "vdir", feature = "caldav", feature = "pimdir", feature = "gcal", feature = "msgraph")`, written out at each site): `BackendCheck::from` and its `code_of` import (src/account/check.rs), `std::path::PathBuf` (src/config.rs) and the tests' `home`, which reads `HOME` through `std::env::var` inline, the tests importing `PathBuf` themselves.
- Allowed in the build with no backend alone, `cfg_attr(not(any(..)), allow(..))` on the item: `unused_variables` once on src/shared/client.rs, as Cardamum's client, the module doc saying why; `dead_code` on `CalendarItemQuery::range` and `kind` (src/shared/item.rs), as `uid` already is outside pimdir; `unused_mut` on the `checks` of `account check` and the `backends` of `account list`; `unused_variables` on the sample test's account (src/config.rs), every section being skipped there; `clippy::single_element_loop` on the backends' round-trip test (src/backend.rs). The two `Unused` variants, which exist only in that build, take a plain `allow(dead_code)`.
- No configuration type moved: the backend blocks stay compiled with their backend alone, a build ignoring the block of one it leaves out, as "Unknown fields" (config) says. A vdir-only build listed an account whose `caldav` block carried `bogus-key` with vdir alone, before and after; the default build refused the key naming the line, before and after.
- shell.nix: openssl beside pkg-config, in `buildInputs` and `LD_LIBRARY_PATH`, as Himalaya's and Cardamum's, so `nix develop --command cargo clippy --no-default-features --features native-tls,caldav` builds without `PKG_CONFIG_PATH`.
- .github/workflows/tests.yml: a `features` job, Himalaya's and Cardamum's, running clippy `-D warnings` then `cargo test`, both `--locked`, over the eleven sets in one job and one cache; actionlint (with shellcheck) passes on it.
- CONTRIBUTING.md: the build with no backend compiles, and a gate change runs the clippy matrix rather than three `cargo build`s.

No behaviour moved: the commands, outputs and accepted configurations of every build are the same. The build with no backend behaved the same before and after: `account list`, `completion`, `manual` and `json-schema` work, and `account check` and the unified commands fail with "No backend matching `auto` is configured for this account".

Capabilities moved: packaging (cairn/spec/packaging.md: "One cargo feature per backend" modified, a build with no backend compiling; "Every backend builds alone" added).

Verified: `cargo fmt --check`; clippy `--all-targets --locked -D warnings` clean over the eleven sets; `cargo test --locked` passing over them with the same counts before and after (default 277, `--all-features` 277, `rustls-ring` 166, `rustls-ring,vdir` 182, `rustls-ring,caldav` 178, `rustls-ring,gcal` 177, `rustls-ring,msgraph` 170, `rustls-ring,pimdir` 236, `native-tls,caldav` 178, `rustls-aws,caldav` 178, `rustls-ring,vdir,pimdir` 253, unit and integration tests together); `json-schema -d` writing the same 48 schemas from the default and the all-features builds before and after.

Left: the CONTRIBUTING feature table, stale on gcal, msgraph and the default set, is release-polish's. actionlint flags .github/workflows/release-on-demand.yml, untouched here: its `os` input defaults to `ubuntu-latest`, which its options leave out.
