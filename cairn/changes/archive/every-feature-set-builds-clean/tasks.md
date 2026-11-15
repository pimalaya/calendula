---
cairn: tasks
change: every-feature-set-builds-clean
---

# Tasks

- [x] Baseline: clippy `--all-targets -D warnings` and `cargo test` over the matrix (default, `--all-features`, `rustls-ring` alone, `rustls-ring` with each of `vdir`, `caldav`, `gcal`, `msgraph`, `pimdir`, `native-tls,caldav`, `rustls-aws,caldav`, `rustls-ring,vdir,pimdir`), errors counted per set
- [x] shell.nix: openssl beside pkg-config, as Himalaya's; `native-tls,caldav` built in `nix develop` alone
- [x] The build with no backend clippy-clean: src/shared/client.rs, src/shared/item.rs, src/account/check.rs, src/account/list.rs, src/config.rs, src/backend.rs
- [x] Clippy and tests clean over the whole matrix; rustfmt
- [x] A vdir-only build ignoring a `caldav` block with an unknown key, a full build refusing it, before and after
- [x] `json-schema -d` of the default and all-features builds identical before and after
- [x] .github/workflows/tests.yml: the `features` job; actionlint
- [x] CONTRIBUTING.md: the build with no backend compiles, the matrix to check
- [x] Fold the delta into cairn/spec/packaging.md; log; tick the-family-standards; status landed; `sh cairn/verify.sh`
